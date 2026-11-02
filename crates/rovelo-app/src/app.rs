//! Application state, commands, undo bookkeeping and the frame loop.
use crate::canvas::{CanvasState, Tool};
use crate::dates::{self, Date};
use crate::ops::{self, Selection};
use crate::panels::{self, brand::BrandUi, code::CodeState, tasks::TasksUi};
use crate::templates::{self, Template};
use crate::{export, files, theme};
use egui::{Context, Key, KeyboardShortcut, Modifiers, RichText};
use rovelo_core::brand::BrandKit;
use rovelo_core::history::History;
use rovelo_core::layout::auto_layout;
use rovelo_core::model::Diagram;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideTab {
    Tasks,
    Brand,
    Code,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cmd {
    New(Template),
    Open,
    Save,
    SaveAs,
    ExportSvg,
    ExportPng,
    CopySvg,
    Undo,
    Redo,
    AutoLayout,
    SetTool(Tool),
    TogglePanel,
    Fit,
    ZoomBy(f32),
    DeleteSelection,
    Duplicate,
    SelectAll,
    Group,
    Nudge(f32, f32),
    Quit,
}

impl Cmd {
    /// Commands that replace the document and so need the unsaved-changes check.
    fn discards(&self) -> bool {
        matches!(self, Cmd::New(_) | Cmd::Open | Cmd::Quit)
    }
}

pub struct Toast {
    pub text: String,
    pub error: bool,
    pub until: Instant,
}

pub struct MermaidModal {
    pub src: String,
}

pub struct RoveloApp {
    pub diagram: Diagram,
    last_seen: Diagram,
    gesture_before: Option<Diagram>,
    pub break_gesture: bool,
    pub history: History,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub sel: Selection,
    pub canvas: CanvasState,
    pub tab: SideTab,
    pub panel_open: bool,
    /// Current appearance, saved so the next launch keeps it.
    pub dark: bool,
    pub tasks_ui: TasksUi,
    pub brand_ui: BrandUi,
    pub code: CodeState,
    pub cmds: Vec<Cmd>,
    pub confirm: Option<Cmd>,
    pub mermaid: Option<MermaidModal>,
    pub toast: Option<Toast>,
    pub default_brand: BrandKit,
    pub today: Date,
    quit_ok: bool,
    dev_select: Option<String>,
    /// ROVELO_THEME forces a theme for this run only; it is never saved.
    theme_forced: bool,
    last_title: String,
}

impl RoveloApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Older builds saved egui's memory (scroll offsets, widget state); start clean of it.
        cc.egui_ctx.memory_mut(|mem| {
            let options = mem.options.clone();
            *mem = egui::Memory::default();
            mem.options = options;
        });
        theme::install(&cc.egui_ctx);
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        // Dark unless the person chose light; the system appearance is never followed.
        let dark = cc
            .storage
            .and_then(|s| eframe::get_value::<bool>(s, "dark"))
            .unwrap_or(true);
        cc.egui_ctx.set_theme(if dark {
            egui::ThemePreference::Dark
        } else {
            egui::ThemePreference::Light
        });
        // New diagrams always start from the house kit.
        let default_brand = BrandKit::default();
        let today = dates::today();
        let diagram = templates::build(Template::Blank, today, &default_brand);
        let mut app = RoveloApp::with_diagram(diagram, default_brand);
        // Developer shortcuts for screenshots: ROVELO_TEMPLATE=user|task, ROVELO_TAB=brand|code.
        match std::env::var("ROVELO_TEMPLATE").as_deref() {
            Ok("user") => app.cmds.push(Cmd::New(Template::UserFlow)),
            Ok("task") => app.cmds.push(Cmd::New(Template::TaskFlow)),
            _ => {}
        }
        match std::env::var("ROVELO_THEME").as_deref() {
            Ok("light") => cc.egui_ctx.set_theme(egui::ThemePreference::Light),
            Ok("dark") => cc.egui_ctx.set_theme(egui::ThemePreference::Dark),
            _ => {}
        }
        match std::env::var("ROVELO_TAB").as_deref() {
            Ok("brand") => app.tab = SideTab::Brand,
            Ok("code") => app.tab = SideTab::Code,
            _ => {}
        }
        app
    }

    pub fn with_diagram(diagram: Diagram, default_brand: BrandKit) -> Self {
        RoveloApp {
            last_seen: diagram.clone(),
            diagram,
            gesture_before: None,
            break_gesture: false,
            history: History::default(),
            path: None,
            dirty: false,
            sel: Selection::default(),
            canvas: CanvasState::default(),
            tab: SideTab::Tasks,
            panel_open: true,
            dark: true,
            tasks_ui: TasksUi::default(),
            brand_ui: BrandUi::default(),
            code: CodeState::default(),
            cmds: Vec::new(),
            confirm: None,
            mermaid: None,
            toast: None,
            default_brand,
            today: dates::today(),
            quit_ok: false,
            dev_select: std::env::var("ROVELO_SELECT").ok(),
            theme_forced: matches!(
                std::env::var("ROVELO_THEME").as_deref(),
                Ok("light" | "dark")
            ),
            last_title: String::new(),
        }
    }

    pub fn notify(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            error: false,
            until: Instant::now() + Duration::from_secs(3),
        });
    }

    pub fn fail(&mut self, text: impl Into<String>) {
        self.toast = Some(Toast {
            text: text.into(),
            error: true,
            until: Instant::now() + Duration::from_secs(6),
        });
    }

    pub fn doc_name(&self) -> String {
        files::doc_name(self.path.as_deref(), &self.diagram.title)
    }

    /// Swap in a whole new document (new, open): history and selection start over.
    pub fn load_document(&mut self, d: Diagram, path: Option<PathBuf>) {
        self.last_seen = d.clone();
        self.diagram = d;
        self.path = path;
        self.dirty = false;
        self.gesture_before = None;
        self.history.clear();
        self.sel.clear();
        self.canvas.reset_for_new_document();
        self.code.mark_diagram_changed();
    }

    /// Swap the diagram but keep history (undo/redo, Mermaid import).
    fn swap_diagram(&mut self, d: Diagram) {
        self.last_seen = d.clone();
        self.diagram = d;
        self.gesture_before = None;
        self.dirty = true;
        self.sel.prune(&self.diagram);
        self.code.mark_diagram_changed();
    }

    /// Replace the diagram as one undoable step.
    pub fn replace_undoable(&mut self, d: Diagram) {
        self.history.commit(self.diagram.clone());
        self.swap_diagram(d);
    }

    /// Run one atomic edit: it becomes exactly one undo step, after any gesture in flight.
    pub fn edit(&mut self, f: impl FnOnce(&mut Diagram, &mut Selection)) {
        self.flush_gesture();
        let before = self.diagram.clone();
        f(&mut self.diagram, &mut self.sel);
        if self.diagram != before {
            self.history.commit(before);
            self.last_seen = self.diagram.clone();
            self.dirty = true;
            self.code.mark_diagram_changed();
        }
    }

    fn flush_gesture(&mut self) {
        if let Some(b) = self.gesture_before.take()
            && b != self.diagram
        {
            self.history.commit(b);
        }
    }

    pub fn run(&mut self, cmd: Cmd) {
        self.cmds.push(cmd);
    }

    fn run_cmd(&mut self, ctx: &Context, cmd: Cmd) {
        if cmd.discards() && self.dirty {
            self.confirm = Some(cmd);
            return;
        }
        match cmd {
            Cmd::New(t) => {
                let d = templates::build(t, self.today, &self.default_brand);
                self.load_document(d, None);
                self.canvas.fit_request = true;
            }
            Cmd::Open => {
                let Some(p) = files::pick_open_diagram() else {
                    return;
                };
                match files::read_diagram(&p) {
                    Ok(d) => {
                        self.load_document(d, Some(p));
                        self.canvas.fit_request = true;
                    }
                    Err(e) => self.fail(format!("Could not open that file: {e}")),
                }
            }
            Cmd::Save => self.save(false),
            Cmd::SaveAs => self.save(true),
            Cmd::ExportSvg => {
                let svg = rovelo_core::svg::to_svg(&self.diagram);
                if let Some(p) = files::pick_save_export(&self.doc_name(), "svg", "SVG") {
                    match files::write_text(&p, &svg) {
                        Ok(()) => self.notify("Exported SVG"),
                        Err(e) => self.fail(format!("Export failed: {e}")),
                    }
                }
            }
            Cmd::ExportPng => {
                let svg = rovelo_core::svg::to_svg(&self.diagram);
                match export::svg_to_png(&svg, 2.0) {
                    Ok(bytes) => {
                        if let Some(p) = files::pick_save_export(&self.doc_name(), "png", "PNG") {
                            match files::write_bytes(&p, &bytes) {
                                Ok(()) => self.notify("Exported PNG at 2x"),
                                Err(e) => self.fail(format!("Export failed: {e}")),
                            }
                        }
                    }
                    Err(e) => self.fail(format!("PNG export failed: {e}")),
                }
            }
            Cmd::CopySvg => {
                ctx.copy_text(rovelo_core::svg::to_svg(&self.diagram));
                self.notify("SVG copied to the clipboard");
            }
            Cmd::Undo => {
                if let Some(prev) = self.history.undo(&self.diagram) {
                    self.swap_diagram(prev);
                }
            }
            Cmd::Redo => {
                if let Some(next) = self.history.redo(&self.diagram) {
                    self.swap_diagram(next);
                }
            }
            Cmd::AutoLayout => {
                let mut d = self.diagram.clone();
                auto_layout(&mut d);
                if d != self.diagram {
                    self.replace_undoable(d);
                }
                self.canvas.fit_request = true;
            }
            Cmd::SetTool(t) => self.canvas.set_tool(t),
            Cmd::TogglePanel => self.panel_open = !self.panel_open,
            Cmd::Fit => self.canvas.fit_request = true,
            Cmd::ZoomBy(f) => self.canvas.zoom_request = Some(f),
            Cmd::DeleteSelection => {
                if !self.sel.is_empty() {
                    self.edit(|d, sel| {
                        ops::delete_selection(d, sel);
                        sel.clear();
                    });
                }
            }
            Cmd::Duplicate => {
                if !self.sel.nodes.is_empty() {
                    self.edit(|d, sel| {
                        let ids = ops::duplicate(d, &sel.nodes.clone(), (24.0, 24.0));
                        sel.clear();
                        sel.nodes = ids;
                    });
                }
            }
            Cmd::SelectAll => {
                self.sel.clear();
                self.sel.nodes = self.diagram.nodes.iter().map(|n| n.id.clone()).collect();
            }
            Cmd::Group => {
                if !self.sel.nodes.is_empty() {
                    self.edit(|d, sel| {
                        if let Some(g) = ops::group_nodes(d, &sel.nodes.clone()) {
                            sel.only_group(&g);
                        }
                    });
                }
            }
            Cmd::Nudge(dx, dy) => {
                if !self.sel.nodes.is_empty() {
                    self.edit(|d, sel| ops::nudge(d, &sel.nodes.clone(), dx, dy));
                }
            }
            Cmd::Quit => {
                self.quit_ok = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn save(&mut self, as_new: bool) {
        let path = match (&self.path, as_new) {
            (Some(p), false) => Some(p.clone()),
            _ => files::pick_save_diagram(&self.doc_name()),
        };
        let Some(p) = path else { return };
        match files::write_diagram(&p, &self.diagram) {
            Ok(()) => {
                self.path = Some(p);
                self.dirty = false;
                self.notify("Saved");
            }
            Err(e) => self.fail(format!("Could not save: {e}")),
        }
    }

    fn shortcuts(&mut self, ctx: &Context) {
        let typing = ctx.egui_wants_keyboard_input();
        let cs = Modifiers::COMMAND | Modifiers::SHIFT;
        let c = Modifiers::COMMAND;
        let table: &[(Modifiers, Key, bool, Cmd)] = &[
            (cs, Key::Z, true, Cmd::Redo),
            (c, Key::Z, true, Cmd::Undo),
            (cs, Key::S, false, Cmd::SaveAs),
            (c, Key::S, false, Cmd::Save),
            (c, Key::O, false, Cmd::Open),
            (c, Key::E, false, Cmd::ExportSvg),
            (c, Key::D, true, Cmd::Duplicate),
            (c, Key::A, true, Cmd::SelectAll),
            (c, Key::G, true, Cmd::Group),
            (c, Key::Num0, false, Cmd::Fit),
            (c, Key::L, true, Cmd::AutoLayout),
            (c, Key::Backslash, false, Cmd::TogglePanel),
            (Modifiers::NONE, Key::V, true, Cmd::SetTool(Tool::Select)),
            (Modifiers::NONE, Key::N, true, Cmd::SetTool(Tool::AddNode)),
            (Modifiers::NONE, Key::C, true, Cmd::SetTool(Tool::Connect)),
            (c, Key::Equals, false, Cmd::ZoomBy(1.25)),
            (c, Key::Plus, false, Cmd::ZoomBy(1.25)),
            (c, Key::Minus, false, Cmd::ZoomBy(0.8)),
            (Modifiers::NONE, Key::Delete, true, Cmd::DeleteSelection),
            (Modifiers::NONE, Key::Backspace, true, Cmd::DeleteSelection),
            (Modifiers::NONE, Key::ArrowLeft, true, Cmd::Nudge(-8.0, 0.0)),
            (Modifiers::NONE, Key::ArrowRight, true, Cmd::Nudge(8.0, 0.0)),
            (Modifiers::NONE, Key::ArrowUp, true, Cmd::Nudge(0.0, -8.0)),
            (Modifiers::NONE, Key::ArrowDown, true, Cmd::Nudge(0.0, 8.0)),
            (Modifiers::ALT, Key::ArrowLeft, true, Cmd::Nudge(-1.0, 0.0)),
            (Modifiers::ALT, Key::ArrowRight, true, Cmd::Nudge(1.0, 0.0)),
            (Modifiers::ALT, Key::ArrowUp, true, Cmd::Nudge(0.0, -1.0)),
            (Modifiers::ALT, Key::ArrowDown, true, Cmd::Nudge(0.0, 1.0)),
        ];
        let mut found = Vec::new();
        ctx.input_mut(|i| {
            for (m, k, needs_canvas, cmd) in table {
                if *needs_canvas && typing {
                    continue;
                }
                if i.consume_shortcut(&KeyboardShortcut::new(*m, *k)) {
                    found.push(cmd.clone());
                }
            }
        });
        if !typing && ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.sel.clear();
            self.canvas.set_tool(Tool::Select);
        }
        self.cmds.extend(found);
    }

    fn handle_close(&mut self, ctx: &Context) {
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.quit_ok {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm = Some(Cmd::Quit);
        }
    }

    /// Detect edits made this frame and fold each continuous gesture into one undo step.
    fn track_changes(&mut self, ctx: &Context) {
        if self.diagram != self.last_seen {
            if self.gesture_before.is_none() {
                self.gesture_before = Some(self.last_seen.clone());
            }
            self.last_seen = self.diagram.clone();
            self.dirty = true;
            self.code.mark_diagram_changed();
        }
        let busy = ctx.input(|i| i.pointer.any_down()) || ctx.egui_wants_keyboard_input();
        if (!busy || self.break_gesture)
            && let Some(before) = self.gesture_before.take()
            && before != self.diagram
        {
            self.history.commit(before);
        }
        self.break_gesture = false;
        self.sel.prune(&self.diagram);
    }

    fn update_title(&mut self, ctx: &Context) {
        let title = format!(
            "{}{} \u{2014} Rovelo",
            if self.dirty { "\u{25cf} " } else { "" },
            self.doc_name()
        );
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }

    fn confirm_modal(&mut self, ctx: &Context) {
        let Some(cmd) = self.confirm.clone() else {
            return;
        };
        let what = match cmd {
            Cmd::Quit => "Quit Rovelo",
            Cmd::Open => "Open another file",
            _ => "Start a new diagram",
        };
        let mut choice = None;
        let tk = theme::Tokens::from_dark(ctx.global_style().visuals.dark_mode);
        egui::Modal::new(egui::Id::unique("confirm"))
            .frame(theme::panel_frame(&tk).inner_margin(theme::m::px(theme::m::PANEL_PAD_FORM)))
            .show(ctx, |ui| {
                ui.set_width(theme::m::SPACE_32 * 11.0);
                ui.label(theme::title("You have unsaved changes"));
                theme::muted(
                    ui,
                    format!("{what} without saving \u{201c}{}\u{201d}?", self.doc_name()),
                );
                ui.add_space(theme::m::SPACE_16);
                ui.horizontal(|ui| {
                    if theme::primary_button(ui, "Save first").clicked() {
                        choice = Some(2);
                    }
                    if theme::button(ui, "Discard changes").clicked() {
                        choice = Some(1);
                    }
                    if theme::button(ui, "Cancel").clicked() {
                        choice = Some(0);
                    }
                });
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    choice = Some(0);
                }
            });
        match choice {
            Some(1) => {
                self.confirm = None;
                self.dirty = false;
                self.cmds.push(cmd);
            }
            Some(2) => {
                self.confirm = None;
                self.save(false);
                if !self.dirty {
                    self.cmds.push(cmd);
                }
            }
            Some(_) => self.confirm = None,
            None => {}
        }
    }

    fn toast_overlay(&mut self, ctx: &Context) {
        let Some(t) = &self.toast else { return };
        if Instant::now() >= t.until {
            self.toast = None;
            return;
        }
        ctx.request_repaint_after(Duration::from_millis(250));
        let tk = theme::Tokens::from_dark(ctx.global_style().visuals.dark_mode);
        let (text, error) = (t.text.clone(), t.error);
        egui::Area::new(egui::Id::unique("toast"))
            .anchor(
                egui::Align2::CENTER_BOTTOM,
                egui::vec2(0.0, -(theme::m::EDGE + theme::m::PILL_H + theme::m::EDGE)),
            )
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(if error { tk.danger } else { tk.text })
                    .corner_radius(theme::m::r(theme::m::RADIUS_MD))
                    .inner_margin(egui::Margin::symmetric(
                        theme::m::px(theme::m::CONTROL_PAD_X),
                        theme::m::px(theme::m::SPACE_8),
                    ))
                    .show(ui, |ui| {
                        ui.label(RichText::new(text).color(if error {
                            egui::Color32::WHITE
                        } else {
                            tk.surface
                        }));
                    });
            });
    }
}

impl RoveloApp {
    /// One frame of the whole app; `eframe::App::ui` and the headless tests both call it.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.dark = ui.visuals().dark_mode;
        self.today = dates::today();
        self.handle_close(&ctx);
        if self.confirm.is_none() && self.mermaid.is_none() {
            self.shortcuts(&ctx);
        }
        for cmd in std::mem::take(&mut self.cmds) {
            self.run_cmd(&ctx, cmd);
        }
        if self.dev_select.is_some()
            && !self.diagram.nodes.is_empty()
            && let Some(id) = self
                .dev_select
                .take()
                .filter(|i| self.diagram.node(i).is_some())
        {
            self.sel.only_node(&id);
        }
        panels::code::sync(self, &ctx);

        egui::CentralPanel::no_frame().show(ui, |ui| crate::canvas::show(self, ui));
        panels::toolbar::show(self, &ctx);
        panels::side::show(self, &ctx);
        panels::inspector::show(self, &ctx);

        panels::code::mermaid_modal(self, &ctx);
        self.confirm_modal(&ctx);
        self.toast_overlay(&ctx);
        self.track_changes(&ctx);
        self.update_title(&ctx);
    }
}

impl eframe::App for RoveloApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if !self.theme_forced {
            eframe::set_value(storage, "dark", &self.dark);
        }
    }

    /// Scroll offsets and widget state belong to one session, not to the next launch.
    fn persist_egui_memory(&self) -> bool {
        false
    }
}
