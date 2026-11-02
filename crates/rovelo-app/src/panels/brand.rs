use crate::app::RoveloApp;
use crate::theme::{self, Tokens, m, rgb};
use crate::widgets::slider;
use crate::{export, files};
use egui::{FontId, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, pos2, vec2};
use rovelo_core::brand::{BrandKit, EdgeRouting, NodeFill, Rgb, Role};

#[derive(Default)]
pub struct BrandUi {
    logo_text: String,
    logo_editor: bool,
    logo_error: Option<String>,
}

/// A colour well for one brand colour (egui's picker opens on click).
fn color_row(ui: &mut Ui, c: &mut Rgb) -> bool {
    let mut a = [c.0, c.1, c.2];
    let changed = ui
        .scope(|ui| {
            ui.spacing_mut().interact_size = vec2(m::CONTROL_H, m::SWATCH);
            egui::color_picker::color_edit_button_srgb(ui, &mut a).changed()
        })
        .inner;
    if changed {
        *c = Rgb(a[0], a[1], a[2]);
    }
    changed
}

/// One brand number as an Inset slider; the house kit's value is the default.
fn slider(ui: &mut Ui, label: &str, v: &mut f32, min: f32, max: f32, step: f32, default: f32) {
    let spec = slider::Spec {
        label,
        min,
        max,
        step,
        unit: " px",
        default,
    };
    slider::inset_slider(ui, spec, v);
}

/// One preset card: three palette dots and the kit's name; selected is neutral.
fn preset_card(ui: &mut Ui, p: &BrandKit, active: bool, w: f32) -> egui::Response {
    let tk = Tokens::of(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, m::SPACE_32 + m::SPACE_24), Sense::click());
    let fill = if active {
        theme::selected_fill(&tk)
    } else if resp.hovered() {
        tk.sunk
    } else {
        tk.surface
    };
    let stroke = if active {
        theme::selected_border(&tk)
    } else {
        tk.border
    };
    ui.painter().rect(
        rect,
        m::RADIUS_MD,
        fill,
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    for (i, r) in [Role::Primary, Role::Secondary, Role::Accent]
        .iter()
        .enumerate()
    {
        let c = rect.left_top()
            + vec2(
                m::SPACE_12 + m::SPACE_4 + i as f32 * m::SPACE_12,
                m::SPACE_16,
            );
        ui.painter()
            .circle_filled(c, m::SPACE_4 + 1.0, rgb(p.palette.get(*r)));
    }
    let ink = if active {
        theme::selected_ink(&tk)
    } else {
        tk.text
    };
    ui.painter().text(
        pos2(rect.left() + m::SPACE_12, rect.bottom() - m::SPACE_12),
        egui::Align2::LEFT_CENTER,
        &p.name,
        FontId::new(m::TEXT_XS, theme::medium_family()),
        ink,
    );
    theme::focus_ring(ui, &resp, m::RADIUS_MD);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn show(app: &mut RoveloApp, ui: &mut Ui) {
    let RoveloApp {
        diagram,
        brand_ui: bu,
        ..
    } = app;
    let tk = Tokens::of(ui);
    let mut notice: Option<(bool, String)> = None;
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            theme::form(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(m::SPACE_8, m::LABEL_GAP);
                let brand = &mut diagram.brand;

                theme::field(ui, "Presets");
                let presets = BrandKit::presets();
                let cols = 2.0;
                let w = ((ui.available_width() - m::SPACE_8 * (cols - 1.0)) / cols).floor();
                for pair in presets.chunks(2) {
                    ui.horizontal(|ui| {
                        for p in pair {
                            if preset_card(ui, p, brand.name == p.name, w).clicked() {
                                let logo = brand.logo_svg.take();
                                *brand = p.clone();
                                brand.logo_svg = logo;
                            }
                        }
                    });
                }

                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Colours");
                egui::Grid::new("brand_roles")
                    .num_columns(4)
                    .spacing([m::SPACE_8, m::SPACE_8])
                    .show(ui, |ui| {
                        for (i, role) in Role::ALL.iter().enumerate() {
                            let mut c = brand.palette.get(*role);
                            if color_row(ui, &mut c) {
                                brand.palette.set(*role, c);
                            }
                            ui.label(RichText::new(role.name()).size(m::TEXT_SM));
                            if i % 2 == 1 {
                                ui.end_row();
                            }
                        }
                        ui.end_row();
                        color_row(ui, &mut brand.background);
                        ui.label("Background");
                        color_row(ui, &mut brand.surface);
                        ui.label("Surface");
                        ui.end_row();
                        color_row(ui, &mut brand.text);
                        ui.label("Text");
                        color_row(ui, &mut brand.muted_text);
                        ui.label("Muted text");
                        ui.end_row();
                    });

                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Body font");
                theme::input(ui, &mut brand.font_family, "Inter, sans-serif");
                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Heading font");
                theme::input(ui, &mut brand.heading_family, "Inter, sans-serif");
                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                let house = BrandKit::default();
                slider(
                    ui,
                    "Font size",
                    &mut brand.font_size,
                    9.0,
                    28.0,
                    1.0,
                    house.font_size,
                );
                slider(
                    ui,
                    "Corner radius",
                    &mut brand.corner_radius,
                    0.0,
                    32.0,
                    1.0,
                    house.corner_radius,
                );
                slider(
                    ui,
                    "Stroke width",
                    &mut brand.stroke_width,
                    0.5,
                    6.0,
                    0.5,
                    house.stroke_width,
                );

                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Edge routing");
                theme::segmented(
                    ui,
                    &mut brand.edge_routing,
                    &[
                        (EdgeRouting::Straight, "Straight"),
                        (EdgeRouting::Orthogonal, "Orthogonal"),
                        (EdgeRouting::Curved, "Curved"),
                    ],
                );
                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Node fill");
                theme::segmented(
                    ui,
                    &mut brand.node_fill,
                    &[
                        (NodeFill::Soft, "Soft"),
                        (NodeFill::Solid, "Solid"),
                        (NodeFill::Outline, "Outline"),
                    ],
                );
                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::switch(ui, &mut brand.shadow, "Soft shadow under nodes");
                theme::switch(ui, &mut brand.show_grid, "Dot grid in exports");

                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Logo");
                match &brand.logo_svg {
                    Some(s) => theme::muted(
                        ui,
                        format!(
                            "Logo set ({} KB). Drawn top-right in exports.",
                            (s.len() / 1024).max(1)
                        ),
                    ),
                    None => theme::muted(ui, "No logo. Paste SVG code or open an .svg file."),
                }
                ui.horizontal(|ui| {
                    if theme::button(ui, "Paste SVG\u{2026}").clicked() {
                        bu.logo_editor = !bu.logo_editor;
                    }
                    if theme::button(ui, "Open file\u{2026}").clicked()
                        && let Some(p) = files::pick_open_svg()
                    {
                        match files::read_text(&p).and_then(|t| export::validate_svg(&t).map(|_| t))
                        {
                            Ok(t) => {
                                brand.logo_svg = Some(t);
                                bu.logo_error = None;
                            }
                            Err(e) => bu.logo_error = Some(e),
                        }
                    }
                    if theme::button_enabled(ui, brand.logo_svg.is_some(), "Clear").clicked() {
                        brand.logo_svg = None;
                    }
                });
                if bu.logo_editor {
                    ui.add(
                        TextEdit::multiline(&mut bu.logo_text)
                            .code_editor()
                            .hint_text("<svg xmlns=\"http://www.w3.org/2000/svg\" \u{2026}>")
                            .margin(egui::Margin::same(m::px(m::INPUT_PAD_X)))
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                    if theme::button_enabled(ui, !bu.logo_text.trim().is_empty(), "Use this logo")
                        .clicked()
                    {
                        match export::validate_svg(bu.logo_text.trim()) {
                            Ok(()) => {
                                brand.logo_svg = Some(bu.logo_text.trim().to_string());
                                bu.logo_text.clear();
                                bu.logo_editor = false;
                                bu.logo_error = None;
                            }
                            Err(e) => bu.logo_error = Some(e),
                        }
                    }
                }
                if let Some(e) = &bu.logo_error {
                    ui.label(
                        RichText::new(format!("! {e}"))
                            .size(m::TEXT_XS)
                            .color(tk.danger),
                    );
                }

                ui.add_space(m::ROW_GAP - m::LABEL_GAP);
                theme::field(ui, "Brand kit file");
                ui.horizontal(|ui| {
                    if theme::button(ui, "Save kit\u{2026}").clicked()
                        && let Some(p) = files::pick_save_brand(&brand.name)
                    {
                        notice = Some(match files::write_brand(&p, brand) {
                            Ok(()) => (false, "Brand kit saved".into()),
                            Err(e) => (true, format!("Could not save the brand kit: {e}")),
                        });
                    }
                    if theme::button(ui, "Load kit\u{2026}").clicked()
                        && let Some(p) = files::pick_open_brand()
                    {
                        match files::read_brand(&p) {
                            Ok(b) => {
                                *brand = b;
                                notice = Some((false, "Brand kit loaded".into()));
                            }
                            Err(e) => notice = Some((true, format!("Not a brand kit: {e}"))),
                        }
                    }
                });
            });
        });
    if let Some((err, msg)) = notice {
        if err { app.fail(msg) } else { app.notify(msg) }
    }
}
