//! Rovelo chrome: tokens from DESIGN.md, the egui styles built from them, and the shared widgets.
use crate::chrome_icons::{self, Icon};
use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Rect, RichText,
    Sense, Stroke, StrokeKind, TextEdit, TextStyle, Theme, Ui, epaint::Shadow, pos2, vec2,
};
use rovelo_core::brand::Rgb;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct Tokens {
    pub dark: bool,
    pub paper: Color32,
    pub surface: Color32,
    pub sunk: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub dot: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub accent_text: Color32,
    pub warning: Color32,
    pub danger: Color32,
}

pub fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

impl Tokens {
    /// Tailwind zinc neutrals with a fuchsia accent (DESIGN.md, Colour).
    pub fn light() -> Tokens {
        Tokens {
            dark: false,
            paper: hex(0xfafafa),
            surface: hex(0xffffff),
            sunk: hex(0xf4f4f5),
            border: hex(0xe4e4e7),
            border_strong: hex(0xd4d4d8),
            dot: hex(0xd4d4d8),
            text: hex(0x18181b),
            muted: hex(0x71717a),
            accent: hex(0xc026d3),
            accent_soft: hex(0xfdf4ff),
            accent_text: hex(0xa21caf),
            warning: hex(0xd97706),
            danger: hex(0xdc2626),
        }
    }

    pub fn dark() -> Tokens {
        Tokens {
            dark: true,
            paper: hex(0x09090b),
            surface: hex(0x18181b),
            sunk: hex(0x27272a),
            border: hex(0x27272a),
            border_strong: hex(0x3f3f46),
            dot: hex(0x27272a),
            text: hex(0xfafafa),
            muted: hex(0xa1a1aa),
            accent: hex(0xc026d3),
            accent_soft: hex(0x4a044e),
            accent_text: hex(0xf0abfc),
            warning: hex(0xfbbf24),
            danger: hex(0xf87171),
        }
    }

    pub fn of(ui: &Ui) -> Tokens {
        Tokens::from_dark(ui.visuals().dark_mode)
    }

    pub fn from_dark(dark: bool) -> Tokens {
        if dark {
            Tokens::dark()
        } else {
            Tokens::light()
        }
    }

    /// Hairline shadow for pills at rest.
    /// Zinc-950 at `alpha`; shadows are tinted, never pure black.
    pub fn shade(&self, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(24, 24, 27, alpha)
    }

    pub fn hairline(&self) -> Shadow {
        Shadow {
            offset: [0, 1],
            blur: 2,
            spread: 0,
            color: if self.dark {
                Color32::TRANSPARENT
            } else {
                self.shade(13)
            },
        }
    }

    /// Shadow under floating panels and menus.
    pub fn float_shadow(&self) -> Shadow {
        Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: if self.dark {
                Color32::TRANSPARENT
            } else {
                self.shade(20)
            },
        }
    }
}

/// Measurements from DESIGN.md ("Measurements", "Type", "Motion"). Every size in the chrome
/// comes from here; the names mirror the design tokens.
pub mod m {
    // Grid steps.
    pub const SPACE_4: f32 = 4.0;
    pub const SPACE_8: f32 = 8.0;
    pub const SPACE_12: f32 = 12.0;
    pub const SPACE_16: f32 = 16.0;
    pub const SPACE_24: f32 = 24.0;
    pub const SPACE_32: f32 = 32.0;
    // Radii.
    pub const RADIUS_SM: f32 = 4.0;
    pub const RADIUS_MD: f32 = 8.0;
    // Controls.
    pub const CONTROL_H: f32 = 32.0;
    pub const CONTROL_PAD_X: f32 = 12.0;
    pub const ICON: f32 = 16.0;
    pub const ICON_SM: f32 = 14.0;
    pub const ICON_GAP: f32 = 8.0;
    // Floating pills.
    pub const PILL_H: f32 = 40.0;
    pub const PILL_PAD: f32 = 4.0;
    pub const PILL_RADIUS: f32 = 12.0;
    pub const PILL_GAP: f32 = 4.0;
    pub const DIVIDER_H: f32 = 16.0;
    pub const DIVIDER_MARGIN: f32 = 4.0;
    // Floating panels.
    pub const PANEL_RADIUS: f32 = 16.0;
    pub const PANEL_PAD_LIST: f32 = 8.0;
    pub const PANEL_PAD_FORM: f32 = 16.0;
    pub const EDGE: f32 = 16.0;
    pub const PANEL_TOP: f32 = EDGE + PILL_H + EDGE;
    pub const HEADER_H: f32 = 32.0;
    pub const LABEL_GAP: f32 = 8.0;
    pub const ROW_GAP: f32 = 16.0;
    // Segmented control.
    pub const TRACK_H: f32 = 32.0;
    pub const TRACK_PAD: f32 = 2.0;
    pub const TRACK_RADIUS: f32 = 10.0;
    pub const SEGMENT_H: f32 = TRACK_H - 2.0 * TRACK_PAD;
    // Small parts.
    pub const CHIP_H: f32 = 24.0;
    pub const CHIP_PAD_X: f32 = 8.0;
    pub const CHIP_RADIUS: f32 = 6.0;
    pub const CHIP_GAP: f32 = 4.0;
    pub const CHECKBOX: f32 = 16.0;
    pub const CHECKBOX_RADIUS: f32 = RADIUS_SM;
    pub const ROW_MIN_H: f32 = 36.0;
    pub const ROW_PAD: f32 = 8.0;
    pub const ROW_RADIUS: f32 = 8.0;
    pub const ROW_TITLE_GAP: f32 = 12.0;
    pub const INPUT_H: f32 = 32.0;
    pub const INPUT_PAD_X: f32 = 12.0;
    pub const SWATCH: f32 = 24.0;
    pub const SWATCH_RING_GAP: f32 = 2.0;
    pub const CARD_W: f32 = 200.0;
    pub const CARD_H: f32 = 160.0;
    pub const CARD_PAD: f32 = 8.0;
    pub const CARD_RADIUS: f32 = 16.0;
    pub const FOCUS_OFFSET: f32 = 2.0;
    pub const FOCUS_W: f32 = 2.0;
    // Type.
    pub const TEXT_XS: f32 = 12.0;
    pub const TEXT_SM: f32 = 14.0;
    pub const TEXT_MD: f32 = 16.0;
    pub const TEXT_LG: f32 = 24.0;
    pub const MONO: f32 = 13.0;
    // Motion, seconds.
    pub const HOVER_S: f32 = 0.15;
    pub const FADE_S: f32 = 0.2;

    pub fn r(v: f32) -> u8 {
        v as u8
    }

    pub fn px(v: f32) -> i8 {
        v as i8
    }
}

pub fn rgb(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

pub fn medium_family() -> FontFamily {
    FontFamily::Name("Inter-Medium".into())
}

pub fn semibold_family() -> FontFamily {
    FontFamily::Name("Inter-SemiBold".into())
}

/// Medium (500) text at `size`.
pub fn med(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text.into()).font(FontId::new(size, medium_family()))
}

/// text-md, 600: panel and inspector titles.
pub fn title(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).font(FontId::new(m::TEXT_MD, semibold_family()))
}

/// Fill and ink for a selected segment, tab, filter, row, tool, preset or swatch.
/// Neutral for now; one place to change when the state rules are settled.
pub fn selected_fill(t: &Tokens) -> Color32 {
    t.sunk
}

pub fn selected_ink(t: &Tokens) -> Color32 {
    t.text
}

pub fn selected_border(t: &Tokens) -> Color32 {
    t.border_strong
}

fn style_for(theme: Theme) -> egui::Style {
    let t = Tokens::from_dark(theme == Theme::Dark);
    let mut s = egui::Style {
        visuals: if t.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        },
        ..Default::default()
    };
    let r = CornerRadius::same(m::r(m::RADIUS_MD));
    let v = &mut s.visuals;
    v.panel_fill = t.paper;
    v.window_fill = t.surface;
    v.extreme_bg_color = t.sunk;
    v.faint_bg_color = t.sunk;
    v.code_bg_color = t.sunk;
    v.text_edit_bg_color = Some(t.sunk);
    v.override_text_color = Some(t.text);
    v.weak_text_color = Some(t.muted);
    v.window_stroke = Stroke::new(1.0, t.border);
    v.window_corner_radius = CornerRadius::same(m::r(m::PANEL_RADIUS));
    v.menu_corner_radius = CornerRadius::same(m::r(m::PILL_RADIUS));
    v.window_shadow = t.float_shadow();
    v.popup_shadow = t.float_shadow();
    v.hyperlink_color = t.accent_text;
    v.error_fg_color = t.danger;
    v.warn_fg_color = t.warning;
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Circle;
    // Text selection and selected list items: a neutral step with full-contrast ink.
    // Sliders override bg_fill with the solid accent (they are an "on" amount).
    v.selection.bg_fill = t.border_strong;
    v.selection.stroke = Stroke::new(1.0, t.text);
    v.text_cursor.stroke = Stroke::new(1.5, t.accent);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.surface;
    w.noninteractive.weak_bg_fill = t.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.text);
    w.noninteractive.corner_radius = r;

    w.inactive.bg_fill = t.sunk;
    w.inactive.weak_bg_fill = t.surface;
    w.inactive.bg_stroke = Stroke::new(1.0, t.border);
    w.inactive.fg_stroke = Stroke::new(1.0, t.text);
    w.inactive.corner_radius = r;

    w.hovered.bg_fill = t.border;
    w.hovered.weak_bg_fill = t.sunk;
    w.hovered.bg_stroke = Stroke::new(1.0, t.border_strong);
    w.hovered.fg_stroke = Stroke::new(1.0, t.text);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 0.0;

    w.active.bg_fill = t.accent;
    w.active.weak_bg_fill = t.sunk;
    w.active.bg_stroke = Stroke::new(1.0, t.accent);
    w.active.fg_stroke = Stroke::new(1.0, t.text);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;
    w.open = w.hovered;

    s.animation_time = m::HOVER_S;
    s.spacing.item_spacing = vec2(m::SPACE_8, m::SPACE_8);
    s.spacing.button_padding = vec2(m::CONTROL_PAD_X, m::SPACE_4);
    s.spacing.interact_size = vec2(m::CONTROL_H, m::CONTROL_H);
    s.spacing.menu_margin = Margin::same(m::px(m::PILL_PAD));
    s.spacing.window_margin = Margin::same(m::px(m::PANEL_PAD_FORM));
    s.spacing.slider_width = 140.0;
    s.spacing.slider_rail_height = m::SPACE_4;
    s.spacing.combo_width = 120.0;
    s.spacing.icon_width = m::CHECKBOX;
    s.spacing.icon_width_inner = m::SPACE_8;
    s.spacing.scroll.bar_width = m::SPACE_8;
    s.spacing.scroll.floating = true;

    let prop = FontFamily::Proportional;
    s.text_styles = [
        (TextStyle::Small, FontId::new(m::TEXT_XS, medium_family())),
        (TextStyle::Body, FontId::new(m::TEXT_SM, prop.clone())),
        (TextStyle::Button, FontId::new(m::TEXT_SM, medium_family())),
        (
            TextStyle::Heading,
            FontId::new(m::TEXT_MD, semibold_family()),
        ),
        (
            TextStyle::Monospace,
            FontId::new(m::MONO, FontFamily::Monospace),
        ),
    ]
    .into();
    let _ = prop;
    s
}

pub fn install(ctx: &egui::Context) {
    ctx.set_style_of(Theme::Light, style_for(Theme::Light));
    ctx.set_style_of(Theme::Dark, style_for(Theme::Dark));
    install_fonts(ctx);
}

fn first_existing(names: &[&str]) -> Option<Vec<u8>> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dirs = [
        format!("{home}/Library/Fonts"),
        "/Library/Fonts".into(),
        "/System/Library/Fonts".into(),
        "/System/Library/Fonts/Supplemental".into(),
    ];
    for d in &dirs {
        for n in names {
            if let Ok(b) = std::fs::read(format!("{d}/{n}")) {
                return Some(b);
            }
        }
    }
    None
}

/// Inter when installed, else the best system sans; egui's bundled fonts stay as glyph fallbacks.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let fallback: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    fn add(
        fonts: &mut FontDefinitions,
        fallback: &[String],
        key: &str,
        files: &[&str],
        family: FontFamily,
    ) -> bool {
        let Some(bytes) = first_existing(files) else {
            return false;
        };
        fonts
            .font_data
            .insert(key.to_owned(), Arc::new(FontData::from_owned(bytes)));
        let mut list = vec![key.to_owned()];
        list.extend(fallback.iter().cloned());
        fonts.families.insert(family, list);
        true
    }
    let regular = add(
        &mut fonts,
        &fallback,
        "ui-regular",
        &[
            "Inter-Regular.ttf",
            "Inter.ttf",
            "SFNS.ttf",
            "HelveticaNeue.ttc",
            "Helvetica.ttc",
        ],
        FontFamily::Proportional,
    );
    let sys: &[&str] = if regular {
        &[]
    } else {
        &["HelveticaNeue.ttc", "Helvetica.ttc"]
    };
    let med_files: Vec<&str> = ["Inter-Medium.ttf"]
        .iter()
        .copied()
        .chain(sys.iter().copied())
        .collect();
    let semi_files: Vec<&str> = ["Inter-SemiBold.ttf", "Inter-Bold.ttf"]
        .iter()
        .copied()
        .chain(sys.iter().copied())
        .collect();
    if !add(
        &mut fonts,
        &fallback,
        "ui-medium",
        &med_files,
        medium_family(),
    ) {
        fonts.families.insert(
            medium_family(),
            fonts.families[&FontFamily::Proportional].clone(),
        );
    }
    if !add(
        &mut fonts,
        &fallback,
        "ui-semibold",
        &semi_files,
        semibold_family(),
    ) {
        fonts.families.insert(
            semibold_family(),
            fonts.families[&FontFamily::Proportional].clone(),
        );
    }
    ctx.set_fonts(fonts);
}

/// Keyboard-focus ring: 2px accent, offset 2. Egui only focuses on Tab, so `has_focus` is focus-visible.
pub fn focus_ring(ui: &Ui, resp: &egui::Response, radius: f32) {
    if resp.has_focus() {
        let t = Tokens::of(ui);
        ui.painter().rect_stroke(
            resp.rect.expand(m::FOCUS_OFFSET),
            radius + m::FOCUS_OFFSET,
            Stroke::new(m::FOCUS_W, t.accent),
            StrokeKind::Outside,
        );
    }
}

/// Section label: text-xs, 500, muted, sentence case.
pub fn section(ui: &mut Ui, label: &str) {
    let t = Tokens::of(ui);
    ui.add_space(m::SPACE_8);
    ui.label(med(label, m::TEXT_XS).color(t.muted));
}

pub fn muted(ui: &mut Ui, text: impl Into<String>) {
    let t = Tokens::of(ui);
    ui.label(RichText::new(text.into()).size(m::TEXT_XS).color(t.muted));
}

/// A form row's label (text-xs muted).
pub fn field(ui: &mut Ui, label: &str) {
    let t = Tokens::of(ui);
    ui.label(med(label, m::TEXT_XS).color(t.muted));
}

/// Wrap form content so it sits 16 from a panel edge (panel padding 8 + 8 here).
pub fn form<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .inner_margin(Margin::same(m::px(m::PANEL_PAD_FORM - m::PANEL_PAD_LIST)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// The 1 × 16 divider between pill groups (its 4px margins come from the pill gap).
pub fn divider(ui: &mut Ui) {
    let t = Tokens::of(ui);
    let extra = (m::DIVIDER_MARGIN - ui.spacing().item_spacing.x).max(0.0);
    ui.add_space(extra);
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, m::CONTROL_H), Sense::hover());
    ui.add_space(extra);
    let r = Rect::from_center_size(rect.center(), vec2(1.0, m::DIVIDER_H));
    ui.painter().rect_filled(r, 0, t.border);
}

fn hover_fill(ui: &Ui, id: egui::Id, hovered: bool, color: Color32) -> Color32 {
    let k = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), hovered, m::HOVER_S);
    color.gamma_multiply(k)
}

/// A 32px icon button with a 16px icon. `active` marks the current tool.
pub fn icon_button(
    ui: &mut Ui,
    icon: Icon,
    active: bool,
    enabled: bool,
    name: &str,
    shortcut: &str,
) -> egui::Response {
    let t = Tokens::of(ui);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(m::CONTROL_H, m::CONTROL_H), sense);
    if ui.is_rect_visible(rect) {
        let hovered = resp.hovered() && enabled;
        if active {
            ui.painter()
                .rect_filled(rect, m::RADIUS_MD, selected_fill(&t));
        } else {
            ui.painter()
                .rect_filled(rect, m::RADIUS_MD, hover_fill(ui, resp.id, hovered, t.sunk));
        }
        let color = if !enabled {
            t.muted.gamma_multiply(0.45)
        } else if active {
            selected_ink(&t)
        } else if hovered {
            t.text
        } else {
            t.muted
        };
        chrome_icons::paint(ui.painter(), icon, rect.center(), m::ICON, color);
        focus_ring(ui, &resp, m::RADIUS_MD);
    }
    resp.on_hover_ui(|ui| tip(ui, name, shortcut))
}

pub fn tip(ui: &mut Ui, name: &str, shortcut: &str) {
    let t = Tokens::of(ui);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = m::SPACE_8;
        ui.label(RichText::new(name).size(m::TEXT_XS));
        if !shortcut.is_empty() {
            ui.label(RichText::new(shortcut).size(m::TEXT_XS).color(t.muted));
        }
    });
}

/// Accent-filled button: 32 high, radius-md, padding 0 12, optional 16px white icon with an 8 gap.
pub fn primary_button(ui: &mut Ui, label: &str) -> egui::Response {
    primary_button_icon(ui, None, label)
}

pub fn primary_button_icon(ui: &mut Ui, icon: Option<Icon>, label: &str) -> egui::Response {
    let t = Tokens::of(ui);
    let font = FontId::new(m::TEXT_SM, medium_family());
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::WHITE);
    let icon_w = if icon.is_some() {
        m::ICON + m::ICON_GAP
    } else {
        0.0
    };
    let w = (m::CONTROL_PAD_X * 2.0 + icon_w + galley.size().x).ceil();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, m::CONTROL_H), Sense::click());
    let base = t.accent;
    // Hover deepens the fill one step to fuchsia-700.
    let hover = hex(0xa21caf);
    let k = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("hover"), resp.hovered(), m::HOVER_S);
    let fill = lerp_color(base, hover, k);
    ui.painter().rect_filled(rect, m::RADIUS_MD, fill);
    let mut x = rect.left() + m::CONTROL_PAD_X;
    if let Some(i) = icon {
        chrome_icons::paint(
            ui.painter(),
            i,
            pos2(x + m::ICON / 2.0, rect.center().y),
            m::ICON,
            Color32::WHITE,
        );
        x += icon_w;
    }
    ui.painter().galley(
        pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        Color32::WHITE,
    );
    focus_ring(ui, &resp, m::RADIUS_MD);
    resp
}

fn lerp_color(a: Color32, b: Color32, k: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k).round() as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// Secondary button: surface fill, 1px border, 32 high, radius-md, padding 0 12, optional 16px icon.
pub fn text_button(
    ui: &mut Ui,
    icon: Option<Icon>,
    label: &str,
    enabled: bool,
    danger: bool,
) -> egui::Response {
    let t = Tokens::of(ui);
    let ink = if !enabled {
        t.muted.gamma_multiply(0.6)
    } else if danger {
        t.danger
    } else {
        t.text
    };
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        FontId::new(m::TEXT_SM, medium_family()),
        ink,
    );
    let icon_w = if icon.is_some() {
        m::ICON + m::ICON_GAP
    } else {
        0.0
    };
    let w = (m::CONTROL_PAD_X * 2.0 + icon_w + galley.size().x).ceil();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(w, m::CONTROL_H), sense);
    let hovered = resp.hovered() && enabled;
    let fill = lerp_color(
        t.surface,
        t.sunk,
        ui.ctx()
            .animate_bool_with_time(resp.id.with("hover"), hovered, m::HOVER_S),
    );
    let stroke = if hovered { t.border_strong } else { t.border };
    ui.painter().rect(
        rect,
        m::RADIUS_MD,
        fill,
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    let mut x = rect.left() + m::CONTROL_PAD_X;
    if let Some(i) = icon {
        let ic = if danger && enabled {
            t.danger
        } else if hovered {
            t.text
        } else {
            t.muted
        };
        chrome_icons::paint(
            ui.painter(),
            i,
            pos2(x + m::ICON / 2.0, rect.center().y),
            m::ICON,
            ic,
        );
        x += icon_w;
    }
    ui.painter().galley(
        pos2(x, rect.center().y - galley.size().y / 2.0),
        galley,
        ink,
    );
    focus_ring(ui, &resp, m::RADIUS_MD);
    resp
}

pub fn icon_text_button(ui: &mut Ui, icon: Icon, label: &str, danger: bool) -> egui::Response {
    text_button(ui, Some(icon), label, true, danger)
}

pub fn button(ui: &mut Ui, label: &str) -> egui::Response {
    text_button(ui, None, label, true, false)
}

pub fn button_enabled(ui: &mut Ui, enabled: bool, label: &str) -> egui::Response {
    text_button(ui, None, label, enabled, false)
}

/// Single-line input: sunk fill, 1px border, 32 high, radius-md, padding 0 12,
/// accent focus border plus a 3px accent-soft ring.
pub fn input(ui: &mut Ui, text: &mut String, hint: &str) -> egui::Response {
    input_sized(ui, text, hint, ui.available_width())
}

pub fn input_sized(ui: &mut Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    let t = Tokens::of(ui);
    let resp = ui.add(input_edit(text, hint, &t).desired_width(width));
    focus_input_ring(ui, &resp);
    resp
}

/// The shared TextEdit builder behind every input.
pub fn input_edit<'a>(text: &'a mut String, hint: &str, t: &Tokens) -> TextEdit<'a> {
    TextEdit::singleline(text)
        .hint_text(RichText::new(hint).color(t.muted))
        .margin(Margin::symmetric(m::px(m::INPUT_PAD_X), m::px(m::SPACE_4)))
        .align(egui::Align2::LEFT_CENTER)
        .min_size(vec2(0.0, m::INPUT_H))
}

pub fn focus_input_ring(ui: &Ui, resp: &egui::Response) {
    if resp.has_focus() {
        let t = Tokens::of(ui);
        ui.painter().rect_stroke(
            resp.rect.expand(1.5),
            m::RADIUS_MD + 1.5,
            Stroke::new(3.0, t.accent_soft),
            StrokeKind::Outside,
        );
    }
}

/// 16px checkbox, radius 4: solid accent with a white tick when checked.
pub fn checkbox(ui: &mut Ui, checked: &mut bool) -> egui::Response {
    let t = Tokens::of(ui);
    let (rect, mut resp) = ui.allocate_exact_size(vec2(m::CHECKBOX, m::CHECKBOX), Sense::click());
    if resp.clicked() {
        *checked = !*checked;
        resp.mark_changed();
    }
    let (fill, stroke) = if *checked {
        (t.accent, t.accent)
    } else {
        (t.surface, t.border_strong)
    };
    ui.painter().rect(
        rect,
        m::CHECKBOX_RADIUS,
        fill,
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    if *checked {
        chrome_icons::paint(
            ui.painter(),
            Icon::Check,
            rect.center(),
            m::SPACE_12,
            Color32::WHITE,
        );
    }
    focus_ring(ui, &resp, m::CHECKBOX_RADIUS);
    resp
}

/// Equal-width segmented control. Returns true when the value changed.
pub fn segmented<T: Copy + PartialEq>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let with: Vec<(T, Option<Icon>, &str)> = options.iter().map(|(v, l)| (*v, None, *l)).collect();
    segmented_icons(ui, value, &with)
}

/// Track 32 / pad 2 / radius 10; segments 28 high, radius 8; optional 14px icons.
pub fn segmented_icons<T: Copy + PartialEq>(
    ui: &mut Ui,
    value: &mut T,
    options: &[(T, Option<Icon>, &str)],
) -> bool {
    let t = Tokens::of(ui);
    let n = options.len().max(1) as f32;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), m::TRACK_H), Sense::hover());
    ui.painter().rect(
        rect,
        m::TRACK_RADIUS,
        t.sunk,
        Stroke::new(1.0, t.border),
        StrokeKind::Inside,
    );
    let inner = rect.shrink(m::TRACK_PAD);
    let w = inner.width() / n;
    let mut changed = false;
    for (i, (v, icon, label)) in options.iter().enumerate() {
        let seg = Rect::from_min_size(inner.min + vec2(w * i as f32, 0.0), vec2(w, m::SEGMENT_H));
        let id = ui
            .scope_id()
            .with(("segmented", rect.min.x as i32, rect.min.y as i32, i));
        let resp = ui.interact(seg, id, Sense::click());
        let selected = *value == *v;
        if selected {
            // Dark: a zinc-700 chip on the zinc-800 track. Light: a white chip raised by a hairline shadow.
            if t.dark {
                ui.painter().rect_filled(seg, m::RADIUS_MD, t.border_strong);
            } else {
                ui.painter().add(t.hairline().as_shape(seg, m::RADIUS_MD));
                ui.painter().rect_filled(seg, m::RADIUS_MD, t.surface);
            }
        } else {
            ui.painter().rect_filled(
                seg,
                m::RADIUS_MD,
                hover_fill(ui, id, resp.hovered(), t.border),
            );
        }
        let color = if selected { selected_ink(&t) } else { t.muted };
        let font = FontId::new(
            m::TEXT_SM,
            if selected {
                medium_family()
            } else {
                FontFamily::Proportional
            },
        );
        let galley = ui
            .painter()
            .layout_no_wrap((*label).to_owned(), font, color);
        let icon_w = if icon.is_some() {
            m::ICON_SM + m::SPACE_8
        } else {
            0.0
        };
        let left = (seg.center().x - (galley.size().x + icon_w) / 2.0).round();
        if let Some(ic) = icon {
            chrome_icons::paint(
                ui.painter(),
                *ic,
                pos2(left + m::ICON_SM / 2.0, seg.center().y),
                m::ICON_SM,
                color,
            );
        }
        ui.painter().galley(
            pos2(left + icon_w, seg.center().y - galley.size().y / 2.0),
            galley,
            color,
        );
        focus_ring(ui, &resp, m::RADIUS_MD);
        if resp.clicked() && !selected {
            *value = *v;
            changed = true;
        }
    }
    changed
}

pub fn chip(
    ui: &mut Ui,
    text: &str,
    color: Color32,
    border: Color32,
    clickable: bool,
) -> egui::Response {
    chip_icon(ui, None, text, color, border, clickable)
}

/// Chip: 24 high, padding 0 8, radius 6, text-xs, optional 14px icon with a 4 gap.
pub fn chip_icon(
    ui: &mut Ui,
    icon: Option<Icon>,
    text: &str,
    color: Color32,
    border: Color32,
    clickable: bool,
) -> egui::Response {
    let t = Tokens::of(ui);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(m::TEXT_XS, medium_family()),
        color,
    );
    let icon_w = if icon.is_some() {
        m::ICON_SM + m::CHIP_GAP
    } else {
        0.0
    };
    let size = vec2(
        (galley.size().x + icon_w + 2.0 * m::CHIP_PAD_X).ceil(),
        m::CHIP_H,
    );
    let (rect, resp) = ui.allocate_exact_size(
        size,
        if clickable {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let fill = if clickable {
        hover_fill(ui, resp.id, resp.hovered(), t.sunk)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect(
        rect,
        m::CHIP_RADIUS,
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    let left = rect.left() + m::CHIP_PAD_X;
    if let Some(i) = icon {
        chrome_icons::paint(
            ui.painter(),
            i,
            pos2(left + m::ICON_SM / 2.0, rect.center().y),
            m::ICON_SM,
            color,
        );
    }
    ui.painter().galley(
        pos2(left + icon_w, rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
    if clickable {
        focus_ring(ui, &resp, m::CHIP_RADIUS);
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }
    resp
}

/// 24px swatch in a 32 slot; the selected one gets a 2px ring 2 outside it.
pub fn swatch(ui: &mut Ui, color: Color32, selected: bool) -> egui::Response {
    let t = Tokens::of(ui);
    let (rect, resp) = ui.allocate_exact_size(vec2(m::CONTROL_H, m::CONTROL_H), Sense::click());
    let c = rect.center();
    let r = m::SWATCH / 2.0;
    let ring = r + m::SWATCH_RING_GAP + 1.0;
    if selected {
        ui.painter()
            .circle_stroke(c, ring, Stroke::new(2.0, selected_ink(&t)));
    } else if resp.hovered() {
        ui.painter()
            .circle_stroke(c, ring, Stroke::new(1.0, t.border_strong));
    }
    ui.painter().circle(c, r, color, Stroke::new(1.0, t.border));
    focus_ring(ui, &resp, m::RADIUS_MD);
    resp
}

/// Floating pill frame: radius 12, padding 4, hairline (light) or border ring only (dark).
pub fn pill_frame(t: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(t.surface)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(m::r(m::PILL_RADIUS))
        .inner_margin(Margin::same(m::px(m::PILL_PAD)))
        .shadow(t.hairline())
}

/// Floating panel frame: radius 16, list padding 8.
pub fn panel_frame(t: &Tokens) -> egui::Frame {
    egui::Frame::new()
        .fill(t.surface)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(m::r(m::PANEL_RADIUS))
        .inner_margin(Margin::same(m::px(m::PANEL_PAD_LIST)))
        .shadow(t.float_shadow())
}

/// Panel header row: 32 high, title text-md left with an optional text-xs id on the same
/// baseline, a 32px close button right; everything centred on the row's midline.
/// Returns true when close was clicked.
pub fn panel_header(ui: &mut Ui, title_text: &str, id_text: &str) -> bool {
    let t = Tokens::of(ui);
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), m::HEADER_H), Sense::hover());
    let title_g = ui.painter().layout_no_wrap(
        title_text.to_owned(),
        FontId::new(m::TEXT_MD, semibold_family()),
        t.text,
    );
    let left = row.left() + m::PANEL_PAD_FORM - m::PANEL_PAD_LIST;
    let top = (row.center().y - title_g.size().y / 2.0).round();
    let base = top + baseline(&title_g);
    let title_w = title_g.size().x;
    ui.painter().galley(pos2(left, top), title_g, t.text);
    if !id_text.is_empty() {
        let id_g = ui.painter().layout_no_wrap(
            id_text.to_owned(),
            FontId::new(m::TEXT_XS, medium_family()),
            t.muted,
        );
        let y = base - baseline(&id_g);
        ui.painter()
            .galley(pos2(left + title_w + m::SPACE_8, y), id_g, t.muted);
    }
    let close = Rect::from_min_size(
        pos2(row.right() - m::CONTROL_H, row.top()),
        vec2(m::CONTROL_H, m::HEADER_H),
    );
    let mut clicked = false;
    ui.scope_builder(egui::UiBuilder::new().max_rect(close), |ui| {
        clicked = icon_button(ui, Icon::Close, false, true, "Close", "Esc").clicked();
    });
    clicked
}

/// Distance from a single-line galley's top to its text baseline.
fn baseline(g: &egui::Galley) -> f32 {
    g.rows
        .first()
        .and_then(|r| r.row.glyphs.first())
        .map_or(g.size().y * 0.75, |gl| gl.pos.y)
}

/// A combo box that opens with the chevron-down icon instead of egui's triangle.
pub fn combo<'a>(id_salt: impl egui::AsIdSalt) -> egui::ComboBox<'a> {
    egui::ComboBox::from_id_salt(id_salt)
        .height(m::CONTROL_H * 8.0)
        .icon(|ui, rect, visuals, _open| {
            chrome_icons::paint(
                ui.painter(),
                Icon::ChevronDown,
                rect.center(),
                m::ICON_SM,
                visuals.fg_stroke.color,
            );
        })
}

/// On/off switch (32 × 16 track): solid accent with a white knob when on, neutral when off.
pub fn switch(ui: &mut Ui, on: &mut bool, label: &str) -> egui::Response {
    let t = Tokens::of(ui);
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_owned(), FontId::proportional(m::TEXT_SM), t.text);
    let size = vec2(m::SPACE_32 + m::SPACE_8 + galley.size().x, m::CONTROL_H);
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let track = Rect::from_min_size(
        pos2(rect.left(), rect.center().y - m::SPACE_8),
        vec2(m::SPACE_32, m::SPACE_16),
    );
    let (fill, stroke) = if *on {
        (t.accent, t.accent)
    } else {
        (t.sunk, t.border_strong)
    };
    ui.painter().rect(
        track,
        m::SPACE_8,
        fill,
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    let k = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("knob"), *on, m::HOVER_S);
    let knob_x = track.left() + m::SPACE_8 + k * m::SPACE_16;
    let knob = if *on { Color32::WHITE } else { t.surface };
    ui.painter().circle(
        pos2(knob_x, track.center().y),
        6.0,
        knob,
        Stroke::new(1.0, t.shade(40)),
    );
    ui.painter().galley(
        pos2(
            track.right() + m::SPACE_8,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        t.text,
    );
    focus_ring(ui, &resp, m::SPACE_8);
    resp
}

#[cfg(test)]
mod tests {
    use super::m::*;

    #[test]
    fn nested_radii_follow_r_minus_p() {
        // (outer radius, padding, inner radius)
        let pairs = [
            ("pill / control", PILL_RADIUS, PILL_PAD, RADIUS_MD),
            ("panel / row", PANEL_RADIUS, PANEL_PAD_LIST, ROW_RADIUS),
            ("track / segment", TRACK_RADIUS, TRACK_PAD, RADIUS_MD),
            ("template card / preview", CARD_RADIUS, CARD_PAD, RADIUS_MD),
        ];
        for (what, outer, pad, inner) in pairs {
            assert_eq!(inner, outer - pad, "{what}");
        }
    }

    #[test]
    fn heights_nest_too() {
        assert_eq!(PILL_H, CONTROL_H + 2.0 * PILL_PAD);
        assert_eq!(SEGMENT_H + 2.0 * TRACK_PAD, TRACK_H);
        assert_eq!(PANEL_TOP, 72.0);
    }

    #[test]
    fn spacing_sits_on_the_4px_grid() {
        for v in [
            SPACE_4,
            SPACE_8,
            SPACE_12,
            SPACE_16,
            SPACE_24,
            SPACE_32,
            PILL_PAD,
            PILL_GAP,
            DIVIDER_MARGIN,
            PANEL_PAD_LIST,
            PANEL_PAD_FORM,
            EDGE,
            CHIP_PAD_X,
            ROW_PAD,
            INPUT_PAD_X,
            CARD_PAD,
            CONTROL_H,
            PILL_H,
            CHIP_H,
            CHECKBOX,
            SWATCH,
            ROW_TITLE_GAP,
            LABEL_GAP,
            ROW_GAP,
        ] {
            assert_eq!(v % 4.0, 0.0, "{v}");
        }
        assert_eq!(TEXT_SM, 14.0);
    }
}
