//! The "Card" node look and the tinted-glass groups (DESIGN.md, "Diagram nodes and groups").
//! Shared by the SVG exporter and the egui canvas so both draw the same thing.
use crate::brand::{BrandKit, NodeStyle, Rgb, Role};
use crate::model::{Diagram, Node, Shape};

pub const MIN_W: f32 = 192.0;
pub const H_ONE: f32 = 48.0;
pub const H_TWO: f32 = 64.0;
pub const PAD: f32 = 12.0;
pub const RADIUS: f32 = 12.0;
pub const TILE: f32 = 28.0;
pub const TILE_RADIUS: f32 = 8.0;
pub const TILE_GAP: f32 = 12.0;
pub const GLYPH: f32 = 16.0;
pub const TITLE_SIZE: f32 = 14.0;
pub const TITLE_LINE: f32 = 20.0;
pub const SUB_SIZE: f32 = 12.0;
pub const SUB_LINE: f32 = 16.0;
pub const SELECT_GAP: f32 = 3.0;
pub const SELECT_RADIUS: f32 = 15.0;
pub const PORT: f32 = 8.0;

pub const EDGE_WIDTH: f32 = 1.5;
pub const EDGE_RADIUS: f32 = 12.0;
pub const ARROW: f32 = 8.0;
pub const ROLE_EDGE_ALPHA: f32 = 0.8;
pub const DASH: f32 = 4.0;
pub const LABEL_H: f32 = 20.0;
pub const LABEL_PAD_X: f32 = 8.0;
pub const LABEL_RADIUS: f32 = 6.0;
pub const LABEL_SIZE: f32 = 12.0;

pub const GROUP_RADIUS: f32 = 20.0;
pub const GROUP_BORDER: f32 = 1.5;
pub const GROUP_TITLE_SIZE: f32 = 12.0;
pub const GROUP_TITLE_INSET: f32 = 12.0;
pub const GROUP_DOT: f32 = 8.0;
pub const GLASS_TOP: f32 = 0.06;
pub const GLASS_STOP: f32 = 0.4;
pub const HIGHLIGHT: f32 = 0.08;

/// The fuchsia used for the "all done" check (Tailwind fuchsia-600).
pub const DONE: Rgb = Rgb(0xc0, 0x26, 0xd3);

pub fn uses_cards(b: &BrandKit) -> bool {
    b.node_style == NodeStyle::Card
}

/// A kit whose background is dark draws dark cards.
pub fn is_dark(b: &BrandKit) -> bool {
    b.background.luminance() < 0.2
}

/// Neutral inks for cards, edges and labels on a dark or light canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ink {
    pub surface: Rgb,
    pub border: Rgb,
    pub text: Rgb,
    pub muted: Rgb,
    pub edge: Rgb,
    pub label_border: Rgb,
}

pub fn ink(b: &BrandKit, dark: bool) -> Ink {
    if dark {
        Ink {
            surface: Rgb(0x18, 0x18, 0x1b),
            border: Rgb(0x27, 0x27, 0x2a),
            text: Rgb(0xfa, 0xfa, 0xfa),
            muted: Rgb(0xa1, 0xa1, 0xaa),
            edge: Rgb(0x52, 0x52, 0x5b),
            label_border: Rgb(0x3f, 0x3f, 0x46),
        }
    } else {
        Ink {
            surface: b.surface,
            border: Rgb(0xe4, 0xe4, 0xe7),
            text: b.text,
            muted: b.muted_text,
            edge: Rgb(0xa1, 0xa1, 0xaa),
            label_border: Rgb(0xe4, 0xe4, 0xe7),
        }
    }
}

/// A role's vibrant step: lighter on dark canvases (about the 400 step), the kit colour on light ones.
pub fn vibrant(b: &BrandKit, role: Role, dark: bool) -> Rgb {
    let c = b.palette.get(role);
    if dark {
        c.mix(Rgb(255, 255, 255), 0.35)
    } else {
        c
    }
}

/// A group's border: the 400 step on dark, the 500 step on light.
pub fn group_edge(b: &BrandKit, role: Role, dark: bool) -> Rgb {
    let c = b.palette.get(role);
    c.mix(Rgb(255, 255, 255), if dark { 0.35 } else { 0.15 })
}

/// Group fill opacity: 10 % on dark, 8 % on light.
pub fn group_alpha(dark: bool) -> f32 {
    if dark { 0.10 } else { 0.08 }
}

/// The icon word shown in a card's tile: the node's own icon, else its kind.
pub fn icon_word(d: &Diagram, n: &Node) -> Option<String> {
    if let Some(i) = n.icon.as_deref().filter(|i| crate::svg::is_known_icon(i)) {
        return Some(i.to_string());
    }
    let incoming = d.edges.iter().any(|e| e.to == n.id);
    let outgoing = d.edges.iter().any(|e| e.from == n.id);
    let w = match n.shape {
        Shape::Diamond => "branch",
        Shape::Screen => "window",
        Shape::Note | Shape::Document => "note",
        Shape::Pill | Shape::Circle if !incoming => "play",
        Shape::Pill | Shape::Circle if !outgoing => "flag",
        _ => return None,
    };
    Some(w.to_string())
}

/// The second line: to-do progress, and whether every to-do is done.
pub fn subtitle(d: &Diagram, n: &Node) -> Option<(String, bool)> {
    let total = d.tasks_for(&n.id).count();
    if total == 0 {
        return None;
    }
    let done = d.tasks_for(&n.id).filter(|t| t.done).count();
    let noun = if total == 1 { "to-do" } else { "to-dos" };
    Some((format!("{done} of {total} {noun}"), done == total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Task;

    #[test]
    fn subtitle_counts_and_flags_all_done() {
        let mut d = Diagram::default();
        d.ensure_node("a", "A");
        assert_eq!(subtitle(&d, &d.nodes[0].clone()), None);
        let mut t = Task::new("t1", "x");
        t.node = Some("a".into());
        d.tasks.push(t.clone());
        t.id = "t2".into();
        t.done = true;
        d.tasks.push(t);
        assert_eq!(
            subtitle(&d, &d.nodes[0].clone()),
            Some(("1 of 2 to-dos".into(), false))
        );
        d.tasks[0].done = true;
        assert_eq!(
            subtitle(&d, &d.nodes[0].clone()),
            Some(("2 of 2 to-dos".into(), true))
        );
    }

    #[test]
    fn kinds_pick_their_icons() {
        let mut d = Diagram::default();
        d.ensure_node("s", "Start").shape = Shape::Pill;
        d.ensure_node("q", "Ok?").shape = Shape::Diamond;
        d.ensure_node("e", "End").shape = Shape::Pill;
        d.add_edge("s", "q");
        d.add_edge("q", "e");
        let word = |id: &str| icon_word(&d, d.node(id).unwrap());
        assert_eq!(word("s").as_deref(), Some("play"));
        assert_eq!(word("q").as_deref(), Some("branch"));
        assert_eq!(word("e").as_deref(), Some("flag"));
        d.node_mut("q").unwrap().icon = Some("mail".into());
        assert_eq!(icon_word(&d, d.node("q").unwrap()).as_deref(), Some("mail"));
    }

    #[test]
    fn inks_follow_the_canvas() {
        let k = BrandKit::rovelo();
        assert!(!is_dark(&k) && is_dark(&BrandKit::midnight()));
        assert_eq!(ink(&k, true).surface, Rgb(0x18, 0x18, 0x1b));
        assert_eq!(ink(&k, false).surface, k.surface);
        assert_ne!(vibrant(&k, Role::Primary, true), k.palette.primary);
        assert_eq!(vibrant(&k, Role::Primary, false), k.palette.primary);
    }
}
