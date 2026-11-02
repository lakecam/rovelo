//! Floating pills: title (top-left), tools (top-centre), file actions (top-right), zoom (bottom-left).
//! Pills are 40 high with 4 padding and radius 12, so their 32px controls nest at radius 8.
use crate::app::{Cmd, RoveloApp};
use crate::canvas::Tool;
use crate::chrome_icons::Icon;
use crate::theme::{self, Tokens, m};
use egui::{
    Align2, Area, Context, FontId, Id, Order, PopupCloseBehavior, Sense, Stroke, Ui, pos2, vec2,
};
use rovelo_core::model::{DiagramKind, Direction};

fn pill(ctx: &Context, id: &str, anchor: Align2, offset: egui::Vec2, add: impl FnOnce(&mut Ui)) {
    let t = Tokens::from_dark(ctx.global_style().visuals.dark_mode);
    Area::new(Id::unique(id))
        .order(Order::Middle)
        .anchor(anchor, offset)
        .show(ctx, |ui| {
            theme::pill_frame(&t).show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(m::PILL_GAP, 0.0);
                ui.set_height(m::CONTROL_H);
                ui.horizontal_centered(add);
            });
        });
}

/// Rovelo mark: a 32px accent tile (radius-md, so it nests in the pill) with two linked nodes.
fn mark(ui: &mut Ui) {
    let t = Tokens::of(ui);
    let (rect, _) = ui.allocate_exact_size(vec2(m::CONTROL_H, m::CONTROL_H), Sense::hover());
    ui.painter().rect_filled(rect, m::RADIUS_MD, t.accent);
    let c = rect.center();
    let w = egui::Color32::WHITE;
    ui.painter()
        .circle_filled(c + vec2(-m::SPACE_4 - 2.0, m::SPACE_4), 3.0, w);
    ui.painter()
        .circle_filled(c + vec2(m::SPACE_4 + 2.0, -m::SPACE_4), 3.0, w);
    ui.painter().line_segment(
        [c + vec2(-m::SPACE_4, 2.0), c + vec2(m::SPACE_4, -2.0)],
        Stroke::new(2.0, w),
    );
}

/// The diagram title as a 32-high text button (padding 0 12) that opens the settings popover.
fn title_button(ui: &mut Ui, name: &str) -> egui::Response {
    let t = Tokens::of(ui);
    let galley = ui.painter().layout_no_wrap(
        name.to_owned(),
        FontId::new(m::TEXT_SM, theme::medium_family()),
        t.text,
    );
    let text_w = galley.size().x.min(m::SPACE_32 * 7.0);
    let (rect, resp) = ui.allocate_exact_size(
        vec2((text_w + 2.0 * m::CONTROL_PAD_X).ceil(), m::CONTROL_H),
        Sense::click(),
    );
    if resp.hovered() {
        ui.painter().rect_filled(rect, m::RADIUS_MD, t.sunk);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let clip = rect.shrink2(vec2(m::CONTROL_PAD_X, 0.0));
    ui.painter().with_clip_rect(clip).galley(
        pos2(clip.left(), rect.center().y - galley.size().y / 2.0),
        galley,
        t.text,
    );
    theme::focus_ring(ui, &resp, m::RADIUS_MD);
    resp.on_hover_ui(|ui| theme::tip(ui, "Rename and diagram settings", ""))
}

pub fn show(app: &mut RoveloApp, ctx: &Context) {
    let t = Tokens::from_dark(ctx.global_style().visuals.dark_mode);
    let mut cmds: Vec<Cmd> = Vec::new();
    let can_undo = app.history.can_undo();
    let can_redo = app.history.can_redo();
    let has_nodes = !app.diagram.nodes.is_empty();
    let tool = app.canvas.tool;
    let dirty = app.dirty;
    let name = app.doc_name();
    let mut toggle_panel = false;
    let mut set_tool: Option<Tool> = None;
    let edge = vec2(m::EDGE, m::EDGE);

    // ---- title ---------------------------------------------------------
    {
        let diagram = &mut app.diagram;
        let panel_open = app.panel_open;
        pill(ctx, "title_pill", Align2::LEFT_TOP, edge, |ui| {
            mark(ui);
            let resp = title_button(ui, &name);
            egui::Popup::menu(&resp)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .frame(theme::panel_frame(&t).inner_margin(m::px(m::PANEL_PAD_FORM)))
                .gap(m::SPACE_8)
                .show(|ui| {
                    ui.set_width(m::SPACE_32 * 8.0);
                    ui.spacing_mut().item_spacing.y = m::LABEL_GAP;
                    theme::field(ui, "Title");
                    theme::input(ui, &mut diagram.title, "Untitled");
                    ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                    theme::field(ui, "Kind");
                    theme::segmented(
                        ui,
                        &mut diagram.kind,
                        &[
                            (DiagramKind::Flowchart, "Flowchart"),
                            (DiagramKind::UserFlow, "User flow"),
                            (DiagramKind::TaskFlow, "Task flow"),
                        ],
                    );
                    ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                    theme::field(ui, "Flow direction");
                    theme::segmented(
                        ui,
                        &mut diagram.direction,
                        &[
                            (Direction::Down, "Down"),
                            (Direction::Right, "Right"),
                            (Direction::Left, "Left"),
                            (Direction::Up, "Up"),
                        ],
                    );
                    theme::muted(ui, "Direction applies the next time you auto-layout.");
                });
            if dirty {
                let (r, resp) =
                    ui.allocate_exact_size(vec2(m::SPACE_8, m::CONTROL_H), Sense::hover());
                ui.painter().circle_filled(r.center(), 3.0, t.muted);
                resp.on_hover_text("Unsaved changes");
            }
            theme::divider(ui);
            if theme::icon_button(
                ui,
                Icon::Sidebar,
                panel_open,
                true,
                "Toggle side panel",
                "\u{2318}\\",
            )
            .clicked()
            {
                toggle_panel = true;
            }
        });
    }

    // ---- tools ---------------------------------------------------------
    pill(
        ctx,
        "tool_pill",
        Align2::CENTER_TOP,
        vec2(0.0, m::EDGE),
        |ui| {
            if theme::icon_button(ui, Icon::Select, tool == Tool::Select, true, "Select", "V")
                .clicked()
            {
                set_tool = Some(Tool::Select);
            }
            if theme::icon_button(
                ui,
                Icon::AddNode,
                tool == Tool::AddNode,
                true,
                "Add node: click the canvas",
                "N",
            )
            .clicked()
            {
                set_tool = Some(if tool == Tool::AddNode {
                    Tool::Select
                } else {
                    Tool::AddNode
                });
            }
            if theme::icon_button(
                ui,
                Icon::Connect,
                tool == Tool::Connect,
                true,
                "Connect: drag from one node to another",
                "C",
            )
            .clicked()
            {
                set_tool = Some(if tool == Tool::Connect {
                    Tool::Select
                } else {
                    Tool::Connect
                });
            }
            theme::divider(ui);
            if theme::icon_button(ui, Icon::Undo, false, can_undo, "Undo", "\u{2318}Z").clicked() {
                cmds.push(Cmd::Undo);
            }
            if theme::icon_button(ui, Icon::Redo, false, can_redo, "Redo", "\u{21e7}\u{2318}Z")
                .clicked()
            {
                cmds.push(Cmd::Redo);
            }
            theme::divider(ui);
            if theme::icon_button(
                ui,
                Icon::Layout,
                false,
                has_nodes,
                "Auto-layout",
                "\u{2318}L",
            )
            .clicked()
            {
                cmds.push(Cmd::AutoLayout);
            }
            if theme::icon_button(ui, Icon::Fit, false, true, "Zoom to fit", "\u{2318}0").clicked()
            {
                cmds.push(Cmd::Fit);
            }
        },
    );

    // ---- file actions --------------------------------------------------
    pill(
        ctx,
        "file_pill",
        Align2::RIGHT_TOP,
        vec2(-m::EDGE, m::EDGE),
        |ui| {
            let dark = ui.visuals().dark_mode;
            let (icon, name) = if dark {
                (Icon::Sun, "Light appearance")
            } else {
                (Icon::Moon, "Dark appearance")
            };
            if theme::icon_button(ui, icon, false, true, name, "").clicked() {
                ui.ctx().set_theme(if dark {
                    egui::ThemePreference::Light
                } else {
                    egui::ThemePreference::Dark
                });
            }
            theme::divider(ui);
            if theme::icon_button(ui, Icon::Open, false, true, "Open", "\u{2318}O").clicked() {
                cmds.push(Cmd::Open);
            }
            if theme::icon_button(ui, Icon::Save, false, true, "Save", "\u{2318}S").clicked() {
                cmds.push(Cmd::Save);
            }
            theme::divider(ui);
            let resp = theme::primary_button_icon(ui, Some(Icon::Export), "Export");
            egui::Popup::menu(&resp)
                .frame(theme::panel_frame(&t))
                .gap(m::SPACE_8)
                .show(|ui| {
                    ui.set_min_width(m::SPACE_32 * 6.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if menu_item(ui, "SVG\u{2026}", "\u{2318}E") {
                        cmds.push(Cmd::ExportSvg);
                    }
                    if menu_item(ui, "PNG at 2x\u{2026}", "") {
                        cmds.push(Cmd::ExportPng);
                    }
                    if menu_item(ui, "Copy SVG", "") {
                        cmds.push(Cmd::CopySvg);
                    }
                });
        },
    );

    // ---- zoom ----------------------------------------------------------
    let zoom = app.canvas.view.zoom;
    pill(
        ctx,
        "zoom_pill",
        Align2::LEFT_BOTTOM,
        vec2(m::EDGE, -m::EDGE),
        |ui| {
            if theme::icon_button(
                ui,
                Icon::ZoomOut,
                false,
                true,
                "Zoom out",
                "\u{2318}\u{2212}",
            )
            .clicked()
            {
                cmds.push(Cmd::ZoomBy(0.8));
            }
            let (rect, resp) = ui.allocate_exact_size(
                vec2(m::SPACE_32 + m::SPACE_16, m::CONTROL_H),
                Sense::click(),
            );
            if resp.hovered() {
                ui.painter().rect_filled(rect, m::RADIUS_MD, t.sunk);
            }
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                format!("{:.0}%", zoom * 100.0),
                FontId::proportional(m::TEXT_SM),
                t.text,
            );
            theme::focus_ring(ui, &resp, m::RADIUS_MD);
            if resp
                .on_hover_ui(|ui| theme::tip(ui, "Reset to 100%", ""))
                .clicked()
            {
                cmds.push(Cmd::ZoomBy(1.0 / zoom.max(0.01)));
            }
            if theme::icon_button(ui, Icon::ZoomIn, false, true, "Zoom in", "\u{2318}+").clicked() {
                cmds.push(Cmd::ZoomBy(1.25));
            }
            theme::divider(ui);
            if theme::icon_button(ui, Icon::Fit, false, true, "Zoom to fit", "\u{2318}0").clicked()
            {
                cmds.push(Cmd::Fit);
            }
        },
    );

    if toggle_panel {
        app.panel_open = !app.panel_open;
    }
    if let Some(tl) = set_tool {
        app.canvas.set_tool(tl);
    }
    app.cmds.extend(cmds);
}

/// A 32-high menu row (radius-md) with an optional muted shortcut on the right.
fn menu_item(ui: &mut Ui, label: &str, shortcut: &str) -> bool {
    let t = Tokens::of(ui);
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), m::CONTROL_H), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, m::RADIUS_MD, t.sunk);
    }
    ui.painter().text(
        pos2(rect.left() + m::CONTROL_PAD_X, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(m::TEXT_SM),
        t.text,
    );
    if !shortcut.is_empty() {
        ui.painter().text(
            pos2(rect.right() - m::CONTROL_PAD_X, rect.center().y),
            Align2::RIGHT_CENTER,
            shortcut,
            FontId::proportional(m::TEXT_XS),
            t.muted,
        );
    }
    theme::focus_ring(ui, &resp, m::RADIUS_MD);
    let clicked = resp.clicked();
    if clicked {
        ui.close();
    }
    clicked
}
