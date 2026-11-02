//! The user's Nucleo duotone icons (18-unit grid), rasterised with resvg and cached as egui textures.
//! The glyph's second tone keeps its own 0.4 opacity, so one colour still reads as two tones.
use egui::{
    Color32, ColorImage, Context, Id, Painter, Pos2, Rect, TextureHandle, TextureOptions, pos2,
    vec2,
};
use resvg::{tiny_skia, usvg};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Select,
    AddNode,
    Connect,
    Undo,
    Redo,
    Layout,
    Fit,
    Open,
    Save,
    Export,
    Moon,
    Sun,
    Tasks,
    Brand,
    Code,
    ZoomIn,
    ZoomOut,
    Calendar,
    Trash,
    Duplicate,
    Sidebar,
    Close,
    Check,
    Link,
    ChevronDown,
    // Node icon words.
    User,
    Lock,
    Mail,
    Home,
    Cart,
    Done,
    Star,
    Gear,
    Search,
    Bell,
    Card,
    Phone,
    Play,
    Branch,
    Flag,
    Window,
    Note,
}

impl Icon {
    #[cfg(test)]
    pub const ALL: [Icon; 42] = [
        Icon::Select,
        Icon::AddNode,
        Icon::Connect,
        Icon::Undo,
        Icon::Redo,
        Icon::Layout,
        Icon::Fit,
        Icon::Open,
        Icon::Save,
        Icon::Export,
        Icon::Moon,
        Icon::Sun,
        Icon::Tasks,
        Icon::Brand,
        Icon::Code,
        Icon::ZoomIn,
        Icon::ZoomOut,
        Icon::Calendar,
        Icon::Trash,
        Icon::Duplicate,
        Icon::Sidebar,
        Icon::Close,
        Icon::Check,
        Icon::Link,
        Icon::ChevronDown,
        Icon::User,
        Icon::Lock,
        Icon::Mail,
        Icon::Home,
        Icon::Cart,
        Icon::Done,
        Icon::Star,
        Icon::Gear,
        Icon::Search,
        Icon::Bell,
        Icon::Card,
        Icon::Phone,
        Icon::Play,
        Icon::Branch,
        Icon::Flag,
        Icon::Window,
        Icon::Note,
    ];

    fn source(self) -> &'static str {
        macro_rules! svg {
            ($f:literal) => {
                include_str!(concat!("../../../assets/icons/", $f, ".svg"))
            };
        }
        match self {
            Icon::Select => svg!("select"),
            Icon::AddNode => svg!("add-node"),
            Icon::Connect => svg!("connect"),
            Icon::Undo => svg!("undo"),
            Icon::Redo => svg!("redo"),
            Icon::Layout => svg!("auto-layout"),
            Icon::Fit => svg!("fit"),
            Icon::Open => svg!("open"),
            Icon::Save => svg!("save"),
            Icon::Export => svg!("export"),
            Icon::Moon => svg!("moon"),
            Icon::Sun => svg!("sun"),
            Icon::Tasks => svg!("tasks"),
            Icon::Brand => svg!("brand"),
            Icon::Code => svg!("code"),
            Icon::ZoomIn => svg!("zoom-in"),
            Icon::ZoomOut => svg!("zoom-out"),
            Icon::Calendar => svg!("calendar"),
            Icon::Trash => svg!("trash"),
            Icon::Duplicate => svg!("duplicate"),
            Icon::Sidebar => svg!("sidebar"),
            Icon::Close => svg!("close"),
            Icon::Check => svg!("check"),
            Icon::Link => svg!("link"),
            Icon::ChevronDown => svg!("chevron-down"),
            Icon::User => svg!("user"),
            Icon::Lock => svg!("lock"),
            Icon::Mail => svg!("mail"),
            Icon::Home => svg!("home"),
            Icon::Cart => svg!("cart"),
            Icon::Done => svg!("done"),
            Icon::Star => svg!("star"),
            Icon::Gear => svg!("gear"),
            Icon::Search => svg!("search"),
            Icon::Bell => svg!("bell"),
            Icon::Card => svg!("card"),
            Icon::Phone => svg!("phone"),
            Icon::Play => svg!("play"),
            Icon::Branch => svg!("branch"),
            Icon::Flag => svg!("flag"),
            Icon::Window => svg!("window"),
            Icon::Note => svg!("note"),
        }
    }

    /// The glyph for a node's icon word, with the same aliases as the SVG exporter.
    pub fn for_node_word(name: &str) -> Option<Icon> {
        if !rovelo_core::svg::is_known_icon(name) {
            return None;
        }
        let n = name.trim().to_ascii_lowercase();
        Some(match n.as_str() {
            "user" | "person" | "account" | "profile" => Icon::User,
            "lock" => Icon::Lock,
            "mail" | "email" | "envelope" | "inbox" => Icon::Mail,
            "home" | "house" => Icon::Home,
            "cart" | "basket" | "shopping-cart" | "checkout" => Icon::Cart,
            "check" | "tick" | "done" | "ok" => Icon::Done,
            "star" => Icon::Star,
            "gear" | "settings" | "cog" => Icon::Gear,
            "search" | "magnifier" | "find" => Icon::Search,
            "bell" | "notification" | "alert" => Icon::Bell,
            "card" | "credit-card" | "payment" => Icon::Card,
            "phone" | "mobile" | "call" => Icon::Phone,
            "play" => Icon::Play,
            "branch" => Icon::Branch,
            "flag" => Icon::Flag,
            "window" => Icon::Window,
            "note" => Icon::Note,
            _ => return None,
        })
    }
}

/// Rasterise `icon` at `px` square in an opaque `color` (premultiplied RGBA).
pub fn rasterise(icon: Icon, color: Color32, px: u32) -> Option<ColorImage> {
    let px = px.clamp(4, 1024);
    let hex = format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b());
    let mut svg = icon.source().replace("currentColor", &hex);
    // A glyph drawn only in the secondary tone (the cursor) would read as disabled; give it full ink.
    let paths = svg.matches("<path").count();
    let faded = svg.matches(r#"opacity="0."#).count();
    if paths > 0 && faded >= paths {
        svg = svg
            .replace(r#" fill-opacity="0.4""#, "")
            .replace(r#" opacity="0.4""#, "");
    }
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(px, px)?;
    let k = px as f32 / tree.size().width().max(1.0);
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(k, k),
        &mut pixmap.as_mut(),
    );
    Some(ColorImage::from_rgba_premultiplied(
        [px as usize, px as usize],
        pixmap.data(),
    ))
}

type Key = (Icon, [u8; 3], u32);

#[derive(Clone, Default)]
struct Cache(Arc<Mutex<HashMap<Key, TextureHandle>>>);

fn texture(ctx: &Context, icon: Icon, color: Color32, px: u32) -> Option<TextureHandle> {
    let cache = ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Cache>(Id::NULL.with("rovelo-icon-cache"))
            .clone()
    });
    let key = (icon, [color.r(), color.g(), color.b()], px);
    let mut map = cache.0.lock().ok()?;
    if let Some(t) = map.get(&key) {
        return Some(t.clone());
    }
    if map.len() > 600 {
        map.clear();
    }
    let img = rasterise(icon, Color32::from_rgb(color.r(), color.g(), color.b()), px)?;
    let tex = ctx.load_texture(format!("icon-{icon:?}-{px}"), img, TextureOptions::LINEAR);
    map.insert(key, tex.clone());
    Some(tex)
}

/// Paint `icon` centred on `center`, `size` logical px square. Chrome sizes are drawn
/// pixel-exact; zoomed canvas sizes use a coarser raster bucket. Colour alpha fades the glyph.
pub fn paint(painter: &Painter, icon: Icon, center: Pos2, size: f32, color: Color32) {
    if !(size.is_finite() && size >= 2.0) || color.a() == 0 {
        return;
    }
    let ctx = painter.ctx();
    let ppp = ctx.pixels_per_point();
    let scale = ppp.max(2.0);
    let exact = (size * scale).round();
    let px = if exact <= 64.0 {
        exact as u32
    } else {
        ((exact / 16.0).ceil() * 16.0).min(1024.0) as u32
    };
    let Some(tex) = texture(ctx, icon, color, px) else {
        return;
    };
    let snap = |v: f32| (v * ppp).round() / ppp;
    let min = pos2(snap(center.x - size / 2.0), snap(center.y - size / 2.0));
    let rect = Rect::from_min_size(min, vec2(size, size));
    let tint = if color.a() == 255 {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(color.a())
    };
    painter.image(
        tex.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        tint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_rasterises_with_ink() {
        for i in Icon::ALL {
            let img = rasterise(i, Color32::WHITE, 32).unwrap_or_else(|| panic!("{i:?}"));
            assert_eq!(img.size, [32, 32]);
            assert!(
                img.pixels.iter().any(|p| p.a() > 200),
                "{i:?} has solid ink"
            );
        }
    }

    #[test]
    fn duotone_second_layer_stays_translucent() {
        let img = rasterise(Icon::Done, Color32::WHITE, 36).unwrap();
        let translucent = img
            .pixels
            .iter()
            .filter(|p| (90..=115).contains(&p.a()))
            .count();
        let solid = img.pixels.iter().filter(|p| p.a() == 255).count();
        assert!(
            translucent > 100 && solid > 20,
            "translucent {translucent}, solid {solid}"
        );
    }

    #[test]
    fn colour_replaces_current_color() {
        let img = rasterise(Icon::Check, Color32::from_rgb(0xc0, 0x26, 0xd3), 32).unwrap();
        let p = img.pixels.iter().find(|p| p.a() == 255).unwrap();
        assert_eq!((p.r(), p.g(), p.b()), (0xc0, 0x26, 0xd3));
    }

    #[test]
    fn node_words_and_aliases_map_to_glyphs() {
        for w in rovelo_core::svg::ICON_NAMES {
            assert!(Icon::for_node_word(w).is_some(), "{w}");
        }
        assert_eq!(Icon::for_node_word("check"), Some(Icon::Done));
        assert_eq!(Icon::for_node_word("Settings"), Some(Icon::Gear));
        assert_eq!(Icon::for_node_word("unicorn"), None);
    }
}
