//! The floating right inspector. It exists only while something is selected.
//! Panel padding 8, header row 32, body as a form 16 from the panel edge.
use crate::app::{Cmd, RoveloApp};
use crate::chrome_icons::Icon;
use crate::ops;
use crate::shapes;
use crate::theme::{self, Tokens, m, rgb};
use egui::{Align2, Area, Context, Id, Order, Sense, Stroke, TextEdit, Ui, vec2};
use rovelo_core::brand::{BrandKit, Role};
use rovelo_core::model::{Arrow, Id as NodeId, LineStyle, Shape};

pub const PANEL_W: f32 = 280.0;

/// A labelled form row: label, 8, control, then 16 to the next row.
fn row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    theme::field(ui, label);
    ui.add_space(m::LABEL_GAP - ui.spacing().item_spacing.y);
    let r = add(ui);
    ui.add_space(m::ROW_GAP - ui.spacing().item_spacing.y);
    r
}

/// Brand-coloured swatches. `None` means "no role" for edges. Returns a click.
fn role_picker(
    ui: &mut Ui,
    brand: &BrandKit,
    current: Option<Role>,
    allow_none: bool,
) -> Option<Option<Role>> {
    let tk = Tokens::of(ui);
    let mut out = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        if allow_none {
            let (rect, resp) =
                ui.allocate_exact_size(vec2(m::CONTROL_H, m::CONTROL_H), Sense::click());
            let c = rect.center();
            let r = m::SWATCH / 2.0;
            if current.is_none() {
                ui.painter().circle_stroke(
                    c,
                    r + m::SWATCH_RING_GAP + 1.0,
                    Stroke::new(2.0, theme::selected_ink(&tk)),
                );
            }
            ui.painter()
                .circle_stroke(c, r, Stroke::new(1.0, tk.border_strong));
            ui.painter().line_segment(
                [
                    c + vec2(-m::SPACE_4, m::SPACE_4),
                    c + vec2(m::SPACE_4, -m::SPACE_4),
                ],
                Stroke::new(1.0, tk.muted),
            );
            if resp.on_hover_text("Automatic (neutral)").clicked() {
                out = Some(None);
            }
        }
        for r in Role::ALL {
            if theme::swatch(ui, rgb(brand.palette.get(r)), current == Some(r))
                .on_hover_text(r.name())
                .clicked()
            {
                out = Some(Some(r));
            }
        }
    });
    out
}

/// Miniature shapes in the diagram's own colours, each a 40 × 32 cell with radius-md.
/// Shape choices drawn as neutral outlines (zinc fill and border), never in the role colour.
fn shape_picker(ui: &mut Ui, current: Option<Shape>) -> Option<Shape> {
    let tk = Tokens::of(ui);
    let (fill, ink) = if tk.dark {
        (tk.border_strong, tk.muted)
    } else {
        (tk.border, tk.muted)
    };
    let mut out = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(m::SPACE_4, m::SPACE_4);
        for s in Shape::ALL {
            let (rect, resp) = ui
                .allocate_exact_size(vec2(m::SPACE_32 + m::SPACE_4, m::CONTROL_H), Sense::click());
            if current == Some(s) {
                ui.painter().rect(
                    rect,
                    m::RADIUS_MD,
                    theme::selected_fill(&tk),
                    Stroke::new(1.0, theme::selected_border(&tk)),
                    egui::StrokeKind::Inside,
                );
            } else if resp.hovered() {
                ui.painter().rect_filled(rect, m::RADIUS_MD, tk.sunk);
            }
            let inner = rect.shrink2(vec2(m::SPACE_8, m::SPACE_8));
            let pts = shapes::outline(s, inner, 10.0, 0.22);
            shapes::fill(ui.painter(), &pts, fill);
            shapes::stroke_closed(ui.painter(), &pts, Stroke::new(1.0, ink));
            theme::focus_ring(ui, &resp, m::RADIUS_MD);
            if resp.on_hover_text(s.name()).clicked() {
                out = Some(s);
            }
        }
    });
    out
}

pub fn show(app: &mut RoveloApp, ctx: &Context) {
    let open = !app.sel.is_empty();
    let fade = ctx.animate_bool_with_time(Id::unique("inspector_fade"), open, m::FADE_S);
    if !open {
        return;
    }
    let t = Tokens::from_dark(ctx.global_style().visuals.dark_mode);
    let max_h = (ctx.content_rect().height() - m::PANEL_TOP - m::EDGE).max(m::SPACE_32 * 6.0);
    let mut close = false;
    Area::new(Id::unique("inspector"))
        .order(Order::Middle)
        .fade_in(false)
        .anchor(Align2::RIGHT_TOP, vec2(-m::EDGE, m::PANEL_TOP))
        .show(ctx, |ui| {
            ui.set_opacity(fade);
            theme::panel_frame(&t).show(ui, |ui| {
                ui.set_width(PANEL_W - 2.0 * m::PANEL_PAD_LIST);
                ui.spacing_mut().item_spacing = vec2(m::SPACE_8, 0.0);
                let (n, e, g) = (
                    app.sel.nodes.len(),
                    app.sel.edges.len(),
                    app.sel.groups.len(),
                );
                let (title, sub) = match (n, e, g) {
                    (1, 0, 0) => ("Node".to_string(), app.sel.nodes[0].clone()),
                    (n, 0, 0) if n > 1 => (format!("{n} nodes"), String::new()),
                    (0, 1, 0) => ("Edge".to_string(), app.sel.edges[0].clone()),
                    (0, 0, 1) => ("Group".to_string(), app.sel.groups[0].clone()),
                    _ => ("Selection".to_string(), String::new()),
                };
                if theme::panel_header(ui, &title, &sub) {
                    close = true;
                }
                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                // Size the body to its content (measured last frame), up to the window; egui's
                // own shrink-to-fit lags inside a floating area.
                let body_max = max_h - 2.0 * m::PANEL_PAD_LIST - m::HEADER_H - m::SPACE_8;
                let measured_id = Id::unique("inspector_content_h");
                let measured: f32 = ui.data(|d| d.get_temp(measured_id)).unwrap_or(body_max);
                let out = egui::ScrollArea::vertical()
                    .max_height(body_max)
                    .min_scrolled_height(measured.min(body_max))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        theme::form(ui, |ui| {
                            ui.spacing_mut().item_spacing = vec2(m::SPACE_8, 0.0);
                            match (n, e, g) {
                                (1, 0, 0) => node(app, ui),
                                (n, 0, 0) if n > 1 => multi(app, ui),
                                (0, 1, 0) => edge(app, ui),
                                (0, 0, 1) => group(app, ui),
                                _ => {
                                    theme::muted(ui, format!("{} items selected", n + e + g));
                                    ui.add_space(m::ROW_GAP);
                                    actions(app, ui);
                                }
                            }
                        });
                    });
                let h = out.content_size.y;
                if (h - measured).abs() > 0.5 {
                    ui.data_mut(|d| d.insert_temp(measured_id, h));
                    ui.ctx().request_repaint();
                }
            });
        });
    if close {
        app.sel.clear();
    }
}

fn actions(app: &mut RoveloApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        if !app.sel.nodes.is_empty()
            && theme::icon_text_button(ui, Icon::Duplicate, "Duplicate", false).clicked()
        {
            app.run(Cmd::Duplicate);
        }
        if theme::icon_text_button(ui, Icon::Trash, "Delete", true).clicked() {
            app.run(Cmd::DeleteSelection);
        }
    });
}

fn multiline(ui: &mut Ui, text: &mut String, hint: &str) {
    let tk = Tokens::of(ui);
    let r = ui.add(
        TextEdit::multiline(text)
            .hint_text(egui::RichText::new(hint).color(tk.muted))
            .margin(egui::Margin::symmetric(
                m::px(m::INPUT_PAD_X),
                m::px(m::SPACE_4) + 2,
            ))
            .min_size(vec2(0.0, m::INPUT_H))
            .desired_rows(1)
            .desired_width(ui.available_width()),
    );
    theme::focus_input_ring(ui, &r);
}

fn node(app: &mut RoveloApp, ui: &mut Ui) {
    let RoveloApp {
        diagram: d,
        sel,
        cmds,
        ..
    } = app;
    let id: NodeId = sel.nodes[0].clone();
    let tk = Tokens::of(ui);
    let Some(idx) = d.nodes.iter().position(|n| n.id == id) else {
        return;
    };
    let brand = d.brand.clone();
    let groups: Vec<(NodeId, String)> = d
        .groups
        .iter()
        .map(|g| (g.id.clone(), g.label.clone()))
        .collect();
    {
        let n = &mut d.nodes[idx];
        row(ui, "Label", |ui| multiline(ui, &mut n.label, "Label"));
        if let Some(s) = row(ui, "Shape", |ui| shape_picker(ui, Some(n.shape))) {
            n.shape = s;
        }
        if let Some(Some(r)) = row(ui, "Colour role", |ui| {
            role_picker(ui, &brand, Some(n.role), false)
        }) {
            n.role = r;
        }
        let mut icon = n.icon.clone().unwrap_or_default();
        if row(ui, "Icon", |ui| {
            theme::input(ui, &mut icon, "user, lock, mail, home\u{2026}")
        })
        .changed()
        {
            n.icon = if icon.is_empty() { None } else { Some(icon) };
        }
        let current = n
            .group
            .as_ref()
            .and_then(|g| groups.iter().find(|(i, _)| i == g))
            .map(|(_, l)| l.clone())
            .unwrap_or_else(|| "None".into());
        row(ui, "Group", |ui| {
            theme::combo("node_group")
                .selected_text(current)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    if ui.selectable_label(n.group.is_none(), "None").clicked() {
                        n.group = None;
                    }
                    for (gid, label) in &groups {
                        if ui
                            .selectable_label(n.group.as_deref() == Some(gid.as_str()), label)
                            .clicked()
                        {
                            n.group = Some(gid.clone());
                        }
                    }
                });
        });
    }

    theme::field(ui, "To-dos");
    ui.add_space(m::SPACE_4);
    let mut remove: Option<NodeId> = None;
    let mut any = false;
    for t in d
        .tasks
        .iter_mut()
        .filter(|t| t.node.as_deref() == Some(id.as_str()))
    {
        any = true;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = m::ROW_TITLE_GAP;
            theme::checkbox(ui, &mut t.done);
            ui.add(
                TextEdit::singleline(&mut t.title)
                    .frame(egui::Frame::NONE)
                    .margin(egui::Margin::ZERO)
                    .align(egui::Align2::LEFT_CENTER)
                    .min_size(vec2(0.0, m::CONTROL_H))
                    .desired_width(ui.available_width() - m::CONTROL_H - m::ROW_TITLE_GAP)
                    .text_color(if t.done { tk.muted } else { tk.text }),
            );
            if theme::icon_button(ui, Icon::Trash, false, true, "Delete to-do", "").clicked() {
                remove = Some(t.id.clone());
            }
        });
    }
    if !any {
        theme::muted(ui, "Nothing to do here yet.");
    }
    if let Some(r) = remove {
        d.tasks.retain(|t| t.id != r);
    }
    ui.add_space(m::SPACE_8);
    let new_id = Id::unique(("node_new_task", &id));
    let mut text: String = ui.data_mut(|mm| mm.get_temp(new_id).unwrap_or_default());
    let mut add = false;
    ui.horizontal(|ui| {
        let add_w = m::SPACE_32 + 2.0 * m::CONTROL_PAD_X;
        let r = theme::input_sized(
            ui,
            &mut text,
            "Add a to-do\u{2026}",
            ui.available_width() - add_w - m::SPACE_8,
        );
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            add = true;
            r.request_focus();
        }
        if theme::button_enabled(ui, !text.trim().is_empty(), "Add").clicked() {
            add = true;
        }
    });
    if add && ops::add_task(d, &text, Some(&id)).is_some() {
        text.clear();
    }
    ui.data_mut(|mm| mm.insert_temp(new_id, text));

    ui.add_space(m::ROW_GAP);
    ui.horizontal(|ui| {
        if theme::icon_text_button(ui, Icon::Duplicate, "Duplicate", false).clicked() {
            cmds.push(Cmd::Duplicate);
        }
        if theme::icon_text_button(ui, Icon::Trash, "Delete", true).clicked() {
            cmds.push(Cmd::DeleteSelection);
        }
    });
}

fn multi(app: &mut RoveloApp, ui: &mut Ui) {
    let brand = app.diagram.brand.clone();
    if let Some(s) = row(ui, "Shape for all", |ui| shape_picker(ui, None)) {
        for id in app.sel.nodes.clone() {
            if let Some(n) = app.diagram.node_mut(&id) {
                n.shape = s;
            }
        }
    }
    if let Some(Some(r)) = row(ui, "Colour role for all", |ui| {
        role_picker(ui, &brand, None, false)
    }) {
        for id in app.sel.nodes.clone() {
            if let Some(n) = app.diagram.node_mut(&id) {
                n.role = r;
            }
        }
    }
    actions(app, ui);
}

fn edge(app: &mut RoveloApp, ui: &mut Ui) {
    let RoveloApp {
        diagram: d,
        sel,
        cmds,
        canvas,
        ..
    } = app;
    let id = sel.edges[0].clone();
    let Some(idx) = d.edges.iter().position(|e| e.id == id) else {
        return;
    };
    let brand = d.brand.clone();
    let name_of = |i: &str| {
        d.node(i)
            .map(|n| n.label.clone())
            .unwrap_or_else(|| i.to_string())
    };
    let (from, to) = (name_of(&d.edges[idx].from), name_of(&d.edges[idx].to));
    theme::muted(ui, format!("{from}  \u{2192}  {to}"));
    ui.add_space(m::ROW_GAP);
    let e = &mut d.edges[idx];
    row(ui, "Label", |ui| theme::input(ui, &mut e.label, "e.g. Yes"));
    row(ui, "Line", |ui| {
        theme::segmented(
            ui,
            &mut e.line,
            &[
                (LineStyle::Solid, "Solid"),
                (LineStyle::Dashed, "Dashed"),
                (LineStyle::Dotted, "Dotted"),
            ],
        )
    });
    row(ui, "Arrowheads", |ui| {
        theme::segmented(
            ui,
            &mut e.arrow,
            &[
                (Arrow::End, "Forward"),
                (Arrow::Both, "Both"),
                (Arrow::None, "None"),
            ],
        )
    });
    if let Some(r) = row(ui, "Colour", |ui| role_picker(ui, &brand, e.role, true)) {
        e.role = r;
    }
    ui.horizontal(|ui| {
        if theme::button(ui, "Reverse").clicked() {
            std::mem::swap(&mut e.from, &mut e.to);
        }
        if theme::button(ui, "Edit label").clicked() {
            canvas.editing = Some(crate::canvas::Editing {
                target: crate::canvas::EditTarget::Edge(id.clone()),
                fresh: true,
            });
        }
    });
    ui.add_space(m::SPACE_8);
    if theme::icon_text_button(ui, Icon::Trash, "Delete", true).clicked() {
        cmds.push(Cmd::DeleteSelection);
    }
}

fn group(app: &mut RoveloApp, ui: &mut Ui) {
    let RoveloApp {
        diagram: d, sel, ..
    } = app;
    let id = sel.groups[0].clone();
    let Some(idx) = d.groups.iter().position(|g| g.id == id) else {
        return;
    };
    let brand = d.brand.clone();
    let members = d
        .nodes
        .iter()
        .filter(|n| n.group.as_deref() == Some(id.as_str()))
        .count();
    {
        let g = &mut d.groups[idx];
        row(ui, "Label", |ui| {
            theme::input(ui, &mut g.label, "Group name")
        });
        if let Some(Some(r)) = row(ui, "Colour role", |ui| {
            role_picker(ui, &brand, Some(g.role), false)
        }) {
            g.role = r;
        }
    }
    theme::muted(ui, format!("{members} nodes inside"));
    ui.add_space(m::SPACE_8);
    if theme::button(ui, "Ungroup")
        .on_hover_text("Keeps the nodes, removes the box")
        .clicked()
    {
        d.remove_group(&id);
        sel.clear();
    }
}
