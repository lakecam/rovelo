//! Node outlines and the built-in icon glyphs for the SVG exporter.
use crate::geometry::{DOC_WAVE, shape_polygon};
use crate::model::{Rect, Shape};
use std::fmt::Write;

pub fn num(v: f32) -> String {
    if !v.is_finite() {
        return "0".into();
    }
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" || s.is_empty() {
        "0".into()
    } else {
        s.into()
    }
}

/// Fill, stroke and extras shared by every outline.
pub struct Paint {
    pub fill: String,
    pub stroke: String,
    pub stroke_width: f32,
    /// Extra attributes such as a filter reference (leading space included, or empty).
    pub extra: &'static str,
}

impl Paint {
    fn attrs(&self) -> String {
        format!(
            r#" fill="{}" stroke="{}" stroke-width="{}" stroke-linejoin="round"{}"#,
            self.fill,
            self.stroke,
            num(self.stroke_width),
            self.extra
        )
    }
}

fn cylinder_cap(h: f32) -> f32 {
    (h * 0.14).clamp(6.0, 14.0)
}

/// Where the label's centre sits in a node, relative to the rectangle's centre.
pub fn text_offset(shape: Shape, r: Rect) -> f32 {
    match shape {
        Shape::Cylinder => cylinder_cap(r.h) * 0.5,
        Shape::Document => -DOC_WAVE * 0.6,
        _ => 0.0,
    }
}

pub fn outline(shape: Shape, r: Rect, radius: f32, p: &Paint) -> String {
    let (x, y, w, h) = (r.x, r.y, r.w, r.h);
    let a = p.attrs();
    let mut s = String::new();
    match shape {
        Shape::Rect => {
            let rx = (radius * 0.3).min(4.0);
            let _ = write!(
                s,
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}"{a}/>"#,
                num(x),
                num(y),
                num(w),
                num(h),
                num(rx)
            );
        }
        Shape::Rounded | Shape::Pill => {
            let rx = if shape == Shape::Pill {
                h / 2.0
            } else {
                radius.min(h / 2.0).min(w / 2.0)
            };
            let _ = write!(
                s,
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}"{a}/>"#,
                num(x),
                num(y),
                num(w),
                num(h),
                num(rx)
            );
        }
        Shape::Diamond | Shape::Hexagon | Shape::Parallelogram => {
            let pts = shape_polygon(shape, r).unwrap_or_default();
            let list: Vec<String> = pts
                .iter()
                .map(|q| format!("{},{}", num(q.x), num(q.y)))
                .collect();
            let _ = write!(s, r#"<polygon points="{}"{a}/>"#, list.join(" "));
        }
        Shape::Circle => {
            let c = r.center();
            let _ = write!(
                s,
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{a}/>"#,
                num(c.x),
                num(c.y),
                num(w / 2.0),
                num(h / 2.0)
            );
        }
        Shape::Cylinder => {
            let ry = cylinder_cap(h);
            let rx = w / 2.0;
            let _ = write!(
                s,
                r#"<path d="M{x0} {y0} A{rx} {ry} 0 0 1 {x1} {y0} L{x1} {y1} A{rx} {ry} 0 0 1 {x0} {y1} Z"{a}/><path d="M{x0} {y0} A{rx} {ry} 0 0 0 {x1} {y0}" fill="none" stroke="{st}" stroke-width="{sw}"/>"#,
                x0 = num(x),
                x1 = num(x + w),
                y0 = num(y + ry),
                y1 = num(y + h - ry),
                rx = num(rx),
                ry = num(ry),
                st = p.stroke,
                sw = num(p.stroke_width),
            );
        }
        Shape::Document => {
            let wave = DOC_WAVE;
            let _ = write!(
                s,
                r#"<path d="M{} {} L{} {} L{} {} Q{} {} {} {} T{} {} Z"{a}/>"#,
                num(x),
                num(y),
                num(x + w),
                num(y),
                num(x + w),
                num(y + h - wave),
                num(x + 0.75 * w),
                num(y + h - 2.0 * wave),
                num(x + 0.5 * w),
                num(y + h - wave),
                num(x),
                num(y + h - wave),
            );
        }
        Shape::Screen => {
            let rx = (radius * 1.8).clamp(14.0, w / 4.0);
            let c = r.center();
            let _ = write!(
                s,
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}"{a}/>"#,
                num(x),
                num(y),
                num(w),
                num(h),
                num(rx)
            );
            let _ = write!(
                s,
                r#"<rect x="{}" y="{}" width="24" height="4" rx="2" fill="{st}" fill-opacity="0.45"/><rect x="{}" y="{}" width="36" height="4" rx="2" fill="{st}" fill-opacity="0.45"/>"#,
                num(c.x - 12.0),
                num(y + 10.0),
                num(c.x - 18.0),
                num(y + h - 14.0),
                st = p.stroke,
            );
        }
        Shape::Note => {
            let f = 16.0f32.min(h / 3.0).min(w / 3.0);
            let _ = write!(
                s,
                r#"<path d="M{} {} L{} {} L{} {} L{} {} L{} {} Z"{a}/><path d="M{} {} L{} {} L{} {} Z" fill="{st}" fill-opacity="0.25" stroke="{st}" stroke-width="{sw}" stroke-linejoin="round"/>"#,
                num(x),
                num(y),
                num(x + w - f),
                num(y),
                num(x + w),
                num(y + f),
                num(x + w),
                num(y + h),
                num(x),
                num(y + h),
                num(x + w - f),
                num(y),
                num(x + w - f),
                num(y + f),
                num(x + w),
                num(y + f),
                st = p.stroke,
                sw = num(p.stroke_width),
            );
        }
    }
    s
}

/// Icon words the exporter can draw.
pub const ICON_NAMES: [&str; 17] = [
    "user", "lock", "mail", "home", "cart", "check", "star", "gear", "search", "bell", "card",
    "phone", "play", "branch", "flag", "window", "note",
];

fn canonical_icon(name: &str) -> Option<&'static str> {
    let n = name.trim().to_ascii_lowercase();
    let n = match n.as_str() {
        "person" | "account" | "profile" => "user",
        "email" | "envelope" | "inbox" => "mail",
        "house" => "home",
        "basket" | "shopping-cart" | "checkout" => "cart",
        "tick" | "done" | "ok" => "check",
        "settings" | "cog" => "gear",
        "magnifier" | "find" => "search",
        "notification" | "alert" => "bell",
        "credit-card" | "payment" => "card",
        "mobile" | "call" => "phone",
        other => return ICON_NAMES.iter().copied().find(|k| *k == other),
    };
    ICON_NAMES.iter().copied().find(|k| *k == n)
}

pub fn is_known_icon(name: &str) -> bool {
    canonical_icon(name).is_some()
}

/// The Nucleo duotone glyph (18-unit grid) behind each icon word.
fn icon_svg(canonical: &str) -> Option<&'static str> {
    Some(match canonical {
        "user" => include_str!("../../../assets/icons/user.svg"),
        "lock" => include_str!("../../../assets/icons/lock.svg"),
        "mail" => include_str!("../../../assets/icons/mail.svg"),
        "home" => include_str!("../../../assets/icons/home.svg"),
        "cart" => include_str!("../../../assets/icons/cart.svg"),
        "check" => include_str!("../../../assets/icons/done.svg"),
        "star" => include_str!("../../../assets/icons/star.svg"),
        "gear" => include_str!("../../../assets/icons/gear.svg"),
        "search" => include_str!("../../../assets/icons/search.svg"),
        "bell" => include_str!("../../../assets/icons/bell.svg"),
        "card" => include_str!("../../../assets/icons/card.svg"),
        "phone" => include_str!("../../../assets/icons/phone.svg"),
        "play" => include_str!("../../../assets/icons/play.svg"),
        "branch" => include_str!("../../../assets/icons/branch.svg"),
        "flag" => include_str!("../../../assets/icons/flag.svg"),
        "window" => include_str!("../../../assets/icons/window.svg"),
        "note" => include_str!("../../../assets/icons/note.svg"),
        _ => return None,
    })
}

/// The markup between `<svg ...>` and `</svg>`.
fn inner_markup(svg: &str) -> Option<&str> {
    let open = svg.find("<svg")?;
    let start = open + svg[open..].find('>')? + 1;
    let end = svg.rfind("</svg>")?;
    (start <= end).then(|| svg[start..end].trim())
}

/// Icon as a `<g>` scaled from the glyph's 18-unit grid to `size`, top-left at (x, y),
/// filled with `color`; the duotone's 0.4-opacity layer is kept. `None` for unknown names.
pub fn icon(name: &str, x: f32, y: f32, size: f32, color: &str) -> Option<String> {
    let mut inner = inner_markup(icon_svg(canonical_icon(name)?)?)?
        .replace("currentColor", color)
        .replace(r#" data-color="color-2""#, "");
    // A glyph drawn only in the secondary tone would look disabled; give it full ink.
    if inner.matches("<path").count() <= inner.matches(r#"opacity="0."#).count() {
        inner = inner
            .replace(r#" fill-opacity="0.4""#, "")
            .replace(r#" opacity="0.4""#, "");
    }
    Some(format!(
        r#"<g transform="translate({} {}) scale({})">{inner}</g>"#,
        num(x),
        num(y),
        num(size / 18.0)
    ))
}
