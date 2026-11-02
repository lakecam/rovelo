use crate::app::{RoveloApp, MermaidModal, SideTab};
use crate::ops;
use crate::theme::{self, Tokens, m};
use egui::{Context, RichText, TextEdit, TextStyle, Ui};
use rovelo_core::layout::{auto_layout, layout_unplaced};
use rovelo_dsl::{Diagnostic, Severity};
use std::time::{Duration, Instant};

const DEBOUNCE: Duration = Duration::from_millis(250);

pub struct CodeState {
    pub text: String,
    /// The text the diagram currently matches.
    applied: String,
    from_diagram: bool,
    edited_at: Option<Instant>,
    pub diagnostics: Vec<Diagnostic>,
    focused: bool,
}

impl Default for CodeState {
    fn default() -> Self {
        CodeState {
            text: String::new(),
            applied: String::new(),
            from_diagram: true,
            edited_at: None,
            diagnostics: vec![],
            focused: false,
        }
    }
}

impl CodeState {
    pub fn mark_diagram_changed(&mut self) {
        self.from_diagram = true;
    }

    pub fn has_pending_edit(&self) -> bool {
        self.text != self.applied
    }
}

/// Keep the text and the diagram in step. Runs every frame, before the panels draw.
pub fn sync(app: &mut RoveloApp, ctx: &Context) {
    if app.tab != SideTab::Code {
        app.code.focused = false;
        return;
    }
    let now = Instant::now();
    if app.code.has_pending_edit() {
        let due = match app.code.edited_at {
            None => {
                app.code.edited_at = Some(now);
                false
            }
            Some(t) => t + DEBOUNCE <= now || !app.code.focused,
        };
        if due {
            apply_text(app);
        } else {
            ctx.request_repaint_after(DEBOUNCE);
        }
        return;
    }
    if app.code.from_diagram && !app.code.focused {
        let s = rovelo_dsl::serialize(&app.diagram);
        app.code.text = s.clone();
        app.code.applied = s;
        app.code.from_diagram = false;
        app.code.edited_at = None;
    }
}

fn apply_text(app: &mut RoveloApp) {
    let parsed = rovelo_dsl::parse(&app.code.text);
    let mut d = ops::merge_parsed(&app.diagram, parsed.diagram);
    layout_unplaced(&mut d);
    app.code.applied = app.code.text.clone();
    app.code.edited_at = None;
    app.code.diagnostics = parsed.diagnostics;
    if d != app.diagram {
        app.diagram = d;
        app.break_gesture = true;
    }
}

pub fn show(app: &mut RoveloApp, ui: &mut Ui) {
    let tk = Tokens::of(ui);
    let pending = app.code.has_pending_edit();
    theme::form(ui, |ui| {
        ui.spacing_mut().item_spacing.y = m::LABEL_GAP;
        ui.horizontal(|ui| {
            if theme::button(ui, "Paste Mermaid\u{2026}")
                .on_hover_text("Import a Mermaid flowchart")
                .clicked()
            {
                app.mermaid = Some(MermaidModal { src: String::new() });
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (txt, c) = if pending {
                    ("Updating\u{2026}", tk.muted)
                } else if app
                    .code
                    .diagnostics
                    .iter()
                    .any(|d| d.severity == Severity::Error)
                {
                    ("! Has errors", tk.danger)
                } else {
                    ("In sync", tk.muted)
                };
                ui.label(theme::med(txt, m::TEXT_XS).color(c));
            });
        });
        ui.add_space(m::ROW_GAP - m::LABEL_GAP);

        let diag_count = app.code.diagnostics.len();
        let line = m::SPACE_24;
        let diag_h = if diag_count == 0 {
            m::SPACE_32 + m::SPACE_16
        } else {
            (diag_count as f32 * line + m::SPACE_32).min(m::SPACE_32 * 5.0)
        };
        let editor_h = (ui.available_height() - diag_h - m::SPACE_24).max(m::SPACE_32 * 3.0);
        let line_h = ui.text_style_height(&TextStyle::Monospace);
        let rows = ((editor_h - 2.0 * m::INPUT_PAD_X) / line_h)
            .floor()
            .max(4.0) as usize;
        let mut changed = false;
        let mut focused = false;
        egui::Frame::new()
            .fill(tk.sunk)
            .stroke(egui::Stroke::new(1.0, tk.border))
            .corner_radius(m::r(m::RADIUS_MD))
            .inner_margin(egui::Margin::same(m::px(m::INPUT_PAD_X)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("code_scroll")
                    .max_height(editor_h - 2.0 * m::INPUT_PAD_X)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        let out = TextEdit::multiline(&mut app.code.text)
                            .code_editor()
                            .frame(egui::Frame::NONE)
                            .desired_width(f32::INFINITY)
                            .desired_rows(rows)
                            .lock_focus(true)
                            .show(ui);
                        changed = out.response.changed();
                        focused = out.response.has_focus();
                    });
            });
        app.code.focused = focused;
        if changed {
            app.code.edited_at = Some(Instant::now());
        }

        theme::section(ui, "Diagnostics");
        if app.code.diagnostics.is_empty() {
            theme::muted(ui, "No problems.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("diag_scroll")
                .max_height(diag_h - m::SPACE_24)
                .show(ui, |ui| {
                    for d in &app.code.diagnostics {
                        let (c, tag) = match d.severity {
                            Severity::Error => (tk.danger, "error"),
                            Severity::Warning => (tk.warning, "warning"),
                            Severity::Info => (tk.muted, "info"),
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = m::SPACE_8;
                            ui.label(
                                RichText::new(format!("line {}", d.line))
                                    .size(m::TEXT_XS)
                                    .color(tk.muted),
                            );
                            ui.label(theme::med(tag, m::TEXT_XS).color(c));
                            ui.label(RichText::new(&d.message).size(m::TEXT_XS));
                        });
                    }
                });
        }
    });
}

pub fn mermaid_modal(app: &mut RoveloApp, ctx: &Context) {
    let Some(mm) = &mut app.mermaid else { return };
    let tk = Tokens::from_dark(ctx.global_style().visuals.dark_mode);
    let mut action = 0;
    egui::Modal::new(egui::Id::unique("mermaid"))
        .frame(theme::panel_frame(&tk).inner_margin(m::px(m::PANEL_PAD_FORM)))
        .show(ctx, |ui| {
            ui.set_width(m::SPACE_32 * 14.0);
            ui.spacing_mut().item_spacing.y = m::LABEL_GAP;
            ui.label(theme::title("Paste a Mermaid flowchart"));
            theme::muted(ui, "It replaces the current diagram. You can undo it.");
            ui.add_space(m::ROW_GAP - m::LABEL_GAP);
            let r = ui.add(
                TextEdit::multiline(&mut mm.src)
                    .code_editor()
                    .hint_text("flowchart LR\n  A[Start] --> B{Ok?}\n  B -->|yes| C[Ship]")
                    .margin(egui::Margin::same(m::px(m::INPUT_PAD_X)))
                    .desired_rows(12)
                    .desired_width(f32::INFINITY),
            );
            if !r.has_focus() && mm.src.is_empty() {
                r.request_focus();
            }
            let preview = rovelo_dsl::from_mermaid(&mm.src);
            let warn = preview.diagnostics.len();
            if !mm.src.trim().is_empty() {
                let notes = if warn > 0 {
                    format!(", {warn} notes")
                } else {
                    String::new()
                };
                theme::muted(
                    ui,
                    format!(
                        "{} nodes, {} edges{notes}",
                        preview.diagram.nodes.len(),
                        preview.diagram.edges.len()
                    ),
                );
                for d in preview.diagnostics.iter().take(4) {
                    theme::muted(ui, format!("line {}: {}", d.line, d.message));
                }
            }
            ui.add_space(m::ROW_GAP - m::LABEL_GAP);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = m::SPACE_8;
                if theme::text_button(ui, None, "Import", !mm.src.trim().is_empty(), false)
                    .clicked()
                {
                    action = 1;
                }
                if theme::button(ui, "Cancel").clicked()
                    || ui.input(|i| i.key_pressed(egui::Key::Escape))
                {
                    action = 2;
                }
            });
        });
    match action {
        1 => {
            if let Some(mm) = app.mermaid.take() {
                let parsed = rovelo_dsl::from_mermaid(&mm.src);
                let mut d = parsed.diagram;
                d.brand = app.diagram.brand.clone();
                auto_layout(&mut d);
                let (n, w) = (d.nodes.len(), parsed.diagnostics.len());
                app.replace_undoable(d);
                app.canvas.fit_request = true;
                app.notify(if w > 0 {
                    format!("Imported {n} nodes with {w} notes")
                } else {
                    format!("Imported {n} nodes")
                });
            }
        }
        2 => app.mermaid = None,
        _ => {}
    }
}
