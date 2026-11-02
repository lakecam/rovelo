//! The Inset slider: one 32-tall row that is the track, label inside left, value inside right,
//! a 16 % ink wash up to a 4 × 20 bar handle that sits 8 in from the ends. Press anywhere sets the
//! value and the pointer carries it (Option = 0.15× fine drag); double-click the number to type,
//! anywhere else to reset. Neutral colours only.
use crate::theme::{Tokens, m, medium_family};
use egui::{CornerRadius, FontFamily, FontId, Key, Rect, Sense, TextEdit, Ui, pos2, vec2};

/// Inset of the handle's travel from each end.
pub const PAD: f32 = 8.0;
/// Text inset from the row's ends.
const TEXT_INSET: f32 = 16.0;
const HANDLE_W: f32 = 4.0;
const HANDLE_H: f32 = 20.0;
const HANDLE_GROW: f32 = 1.25;
const WASH: f32 = 0.16;
const FINE: f32 = 0.15;
/// Two presses closer than this are one double-click; the first one's value is the undo point.
const DOUBLE_S: f64 = 0.4;

// ---- pure parts ----------------------------------------------------------

/// Where the handle sits for `v` in a row `w` wide.
pub fn value_to_x(v: f32, min: f32, max: f32, w: f32) -> f32 {
    let span = (max - min).max(1e-9);
    let k = ((v - min) / span).clamp(0.0, 1.0);
    PAD + k * (w - 2.0 * PAD).max(0.0)
}

/// The value under `x` (relative to the row's left edge), unclamped to the step.
pub fn x_to_value(x: f32, min: f32, max: f32, w: f32) -> f32 {
    let inner = (w - 2.0 * PAD).max(1.0);
    min + (x - PAD) / inner * (max - min)
}

/// Clamp to the range and round to the step.
pub fn snap(v: f32, step: f32, min: f32, max: f32) -> f32 {
    let v = if step > 0.0 {
        (v / step).round() * step
    } else {
        v
    };
    v.clamp(min, max)
}

/// Fine drag: `dx` pixels from the press move the value at 0.15× of the normal rate.
pub fn fine_value(start: f32, dx: f32, min: f32, max: f32, w: f32) -> f32 {
    let inner = (w - 2.0 * PAD).max(1.0);
    start + dx / inner * (max - min) * FINE
}

pub fn decimals(step: f32) -> usize {
    if step >= 1.0 {
        0
    } else if step >= 0.1 {
        1
    } else if step >= 0.01 {
        2
    } else {
        3
    }
}

pub fn format(v: f32, step: f32, unit: &str) -> String {
    format!("{v:.*}{unit}", decimals(step))
}

/// The longest the value reads (the range's longer end).
pub fn widest(min: f32, max: f32, step: f32, unit: &str) -> String {
    let (a, b) = (format(min, step, unit), format(max, step, unit));
    if a.chars().count() > b.chars().count() {
        a
    } else {
        b
    }
}

/// Digits as zeros, the widest digit, so the box never changes width.
pub fn zeros(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_digit() { '0' } else { c })
        .collect()
}

/// True when `v` differs from `default` by more than half a step.
pub fn changed(v: f32, default: f32, step: f32) -> bool {
    (v - default).abs() > (step.max(1e-6) / 2.0)
}

/// Read a typed value: digits, a point and a minus sign only.
pub fn parse_typed(s: &str) -> Option<f32> {
    let digits: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    digits.parse().ok()
}

// ---- widget --------------------------------------------------------------

#[derive(Clone, Default)]
struct State {
    /// Value at the start of the current press; `Some` while dragging.
    start: Option<f32>,
    press_x: f32,
    last_down: f64,
    /// The value before the first click of a double-click.
    undo: Option<f32>,
    editing: bool,
    draft: String,
    fresh: bool,
}

pub struct Spec<'a> {
    pub label: &'a str,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub unit: &'a str,
    pub default: f32,
}

pub fn inset_slider(ui: &mut Ui, spec: Spec<'_>, value: &mut f32) -> egui::Response {
    let t = Tokens::of(ui);
    let w = ui.available_width();
    let (rect, mut resp) = ui.allocate_exact_size(vec2(w, m::CONTROL_H), Sense::click_and_drag());
    let id = resp.id;
    let mut st: State = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    let now = ui.input(|i| i.time);
    let fine = ui.input(|i| i.modifiers.alt);

    let font = FontId::new(m::TEXT_SM, FontFamily::Proportional);
    let shown = format(*value, spec.step, spec.unit);
    let widest_w = ui
        .painter()
        .layout_no_wrap(
            zeros(&widest(spec.min, spec.max, spec.step, spec.unit)),
            font.clone(),
            t.text,
        )
        .size()
        .x;
    let value_rect = Rect::from_min_max(
        pos2(rect.right() - TEXT_INSET - widest_w - 6.0, rect.top()),
        rect.max,
    );
    let before = *value;

    // ---- press / drag ----
    if !st.editing && resp.is_pointer_button_down_on() {
        if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
            let x = p.x - rect.left();
            if st.start.is_none() {
                st.start = Some(*value);
                if now - st.last_down > DOUBLE_S {
                    st.undo = Some(*value);
                }
                st.last_down = now;
                st.press_x = x;
            }
            let start = st.start.unwrap_or(*value);
            let raw = if fine && (x - st.press_x).abs() > 0.0 {
                fine_value(start, x - st.press_x, spec.min, spec.max, w)
            } else {
                x_to_value(x, spec.min, spec.max, w)
            };
            *value = snap(raw, spec.step, spec.min, spec.max);
        }
    } else if st.start.is_some() {
        st.start = None;
    }

    // ---- double-click: type on the number, reset elsewhere ----
    if resp.double_clicked() && !st.editing {
        let on_value = resp
            .interact_pointer_pos()
            .is_some_and(|p| value_rect.contains(p));
        if on_value {
            if let Some(u) = st.undo {
                *value = u;
            }
            st.editing = true;
            st.fresh = true;
            st.draft = format!("{:.*}", decimals(spec.step), *value);
        } else {
            *value = spec.default;
        }
    }

    // ---- paint ----
    let hovered = resp.hovered();
    let dragging = st.start.is_some();
    let lit = hovered || dragging;
    let painter = ui.painter_at(rect.expand(2.0));
    painter.rect_filled(rect, m::RADIUS_MD, t.sunk);
    let x = rect.left() + value_to_x(*value, spec.min, spec.max, w);
    let wash = Rect::from_min_max(rect.min, pos2(x, rect.bottom()));
    let r = m::RADIUS_MD.min(wash.width() / 2.0) as u8;
    painter.rect_filled(
        wash,
        CornerRadius {
            nw: r,
            sw: r,
            ne: 0,
            se: 0,
        },
        t.text.gamma_multiply(WASH),
    );
    let grow = ui
        .ctx()
        .animate_bool_with_time(id.with("grow"), dragging, m::HOVER_S);
    let h = HANDLE_H * (1.0 + (HANDLE_GROW - 1.0) * grow);
    painter.rect_filled(
        Rect::from_center_size(pos2(x, rect.center().y), vec2(HANDLE_W, h)),
        2.0,
        t.text,
    );

    let is_changed = changed(*value, spec.default, spec.step);
    let (label_font, label_ink) = if is_changed {
        (FontId::new(m::TEXT_SM, medium_family()), t.text)
    } else {
        (font.clone(), t.muted)
    };
    let label = painter.layout_no_wrap(spec.label.to_owned(), label_font, label_ink);
    painter.galley(
        pos2(
            rect.left() + TEXT_INSET,
            rect.center().y - label.size().y / 2.0,
        ),
        label,
        label_ink,
    );

    if st.editing {
        let edit_rect = Rect::from_min_max(
            pos2(
                rect.right() - TEXT_INSET - widest_w.max(m::SPACE_32),
                rect.top() + m::SPACE_4,
            ),
            pos2(
                rect.right() - TEXT_INSET + m::SPACE_4,
                rect.bottom() - m::SPACE_4,
            ),
        );
        let mut commit = false;
        let mut cancel = false;
        ui.scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
            let te = TextEdit::singleline(&mut st.draft)
                .font(font.clone())
                .text_color(t.text)
                .frame(egui::Frame::NONE)
                .margin(egui::Margin::ZERO)
                .align(egui::Align2::RIGHT_CENTER)
                .desired_width(edit_rect.width());
            let out = te.show(ui);
            if st.fresh {
                out.response.request_focus();
                st.fresh = false;
            }
            ui.painter().rect_stroke(
                edit_rect.expand2(vec2(m::SPACE_4, 0.0)),
                m::RADIUS_SM,
                egui::Stroke::new(1.0, t.border_strong),
                egui::StrokeKind::Inside,
            );
            if ui.input(|i| i.key_pressed(Key::Escape)) {
                cancel = true;
            } else if out.response.lost_focus() {
                commit = true;
            }
        });
        if cancel {
            st.editing = false;
        } else if commit {
            if let Some(v) = parse_typed(&st.draft) {
                *value = snap(v, spec.step, spec.min, spec.max);
            }
            st.editing = false;
        }
    } else {
        let ink = if lit { t.text } else { t.muted };
        let g = painter.layout_no_wrap(shown, font, ink);
        let right = rect.right() - TEXT_INSET;
        painter.galley(
            pos2(right - g.size().x, rect.center().y - g.size().y / 2.0),
            g,
            ink,
        );
    }

    ui.data_mut(|d| d.insert_temp(id, st));
    if *value != before {
        resp.mark_changed();
    }
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_and_x_map_with_the_8px_pad() {
        assert_eq!(value_to_x(0.0, 0.0, 10.0, 216.0), 8.0);
        assert_eq!(value_to_x(10.0, 0.0, 10.0, 216.0), 208.0);
        assert_eq!(value_to_x(5.0, 0.0, 10.0, 216.0), 108.0);
        assert_eq!(value_to_x(-3.0, 0.0, 10.0, 216.0), 8.0, "clamped");
        assert!((x_to_value(108.0, 0.0, 10.0, 216.0) - 5.0).abs() < 1e-5);
        assert!((x_to_value(8.0, 2.0, 4.0, 116.0) - 2.0).abs() < 1e-5);
        for v in [0.0, 2.5, 7.25, 10.0] {
            let x = value_to_x(v, 0.0, 10.0, 300.0);
            assert!((x_to_value(x, 0.0, 10.0, 300.0) - v).abs() < 1e-4);
        }
    }

    #[test]
    fn step_rounding_and_clamping() {
        assert_eq!(snap(1.26, 0.5, 0.5, 6.0), 1.5);
        assert_eq!(snap(13.4, 1.0, 9.0, 28.0), 13.0);
        assert_eq!(snap(40.0, 1.0, 0.0, 32.0), 32.0);
        assert_eq!(snap(-1.0, 1.0, 0.0, 32.0), 0.0);
    }

    #[test]
    fn fine_drag_is_fifteen_percent() {
        // 200px inner track over a span of 20: a 100px move is 10 normally, 1.5 when fine.
        let v = fine_value(4.0, 100.0, 0.0, 20.0, 216.0);
        assert!((v - 5.5).abs() < 1e-5, "{v}");
    }

    #[test]
    fn fixed_width_string_uses_zeros() {
        assert_eq!(zeros("14.5 px"), "00.0 px");
        assert_eq!(
            widest(0.5, 6.0, 0.5, " px"),
            "6.0 px",
            "ties go to the top end"
        );
        assert_eq!(widest(9.0, 28.0, 1.0, " px"), "28 px");
        assert_eq!(widest(-10.0, 5.0, 1.0, ""), "-10");
        assert_eq!(format(1.5, 0.5, " px"), "1.5 px");
    }

    #[test]
    fn changed_detection_ignores_sub_step_noise() {
        assert!(!changed(10.0, 10.0, 1.0));
        assert!(!changed(10.2, 10.0, 1.0));
        assert!(changed(11.0, 10.0, 1.0));
        assert!(changed(1.0, 1.5, 0.5));
    }

    #[test]
    fn typed_values_are_read_leniently() {
        assert_eq!(parse_typed("12 px"), Some(12.0));
        assert_eq!(parse_typed("1.5"), Some(1.5));
        assert_eq!(parse_typed("abc"), None);
    }
}
