use serde::{Deserialize, Serialize};

/// sRGB colour, `#rrggbb` in text form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn parse(s: &str) -> Option<Rgb> {
        let s = s.trim().trim_start_matches('#');
        let (r, g, b) = match s.len() {
            6 => (
                u8::from_str_radix(&s[0..2], 16).ok()?,
                u8::from_str_radix(&s[2..4], 16).ok()?,
                u8::from_str_radix(&s[4..6], 16).ok()?,
            ),
            3 => {
                let d = |i: usize| u8::from_str_radix(&s[i..=i], 16).ok().map(|v| v * 17);
                (d(0)?, d(1)?, d(2)?)
            }
            _ => return None,
        };
        Some(Rgb(r, g, b))
    }

    /// Mix toward `other` by `t` in 0..=1.
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        let m = |a: u8, b: u8| {
            (a as f32 + (b as f32 - a as f32) * t)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Rgb(m(self.0, other.0), m(self.1, other.1), m(self.2, other.2))
    }

    /// WCAG relative luminance.
    pub fn luminance(self) -> f32 {
        let c = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * c(self.0) + 0.7152 * c(self.1) + 0.0722 * c(self.2)
    }

    pub fn contrast(self, other: Rgb) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Black or white, whichever reads better on this colour.
    pub fn readable_on(self) -> Rgb {
        let white = Rgb(255, 255, 255);
        let black = Rgb(17, 17, 17);
        if self.contrast(white) >= self.contrast(black) {
            white
        } else {
            black
        }
    }
}

/// Named colour slot in a brand kit. Nodes and edges refer to roles, never raw hex,
/// so swapping the kit restyles the whole diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Primary,
    Secondary,
    Accent,
    Neutral,
    Success,
    Warning,
    Danger,
}

impl Role {
    pub const ALL: [Role; 7] = [
        Role::Primary,
        Role::Secondary,
        Role::Accent,
        Role::Neutral,
        Role::Success,
        Role::Warning,
        Role::Danger,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Role::Primary => "primary",
            Role::Secondary => "secondary",
            Role::Accent => "accent",
            Role::Neutral => "neutral",
            Role::Success => "success",
            Role::Warning => "warning",
            Role::Danger => "danger",
        }
    }

    pub fn from_name(s: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.name() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRouting {
    Straight,
    Orthogonal,
    Curved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeFill {
    /// Solid role colour with readable text on top.
    Solid,
    /// Tinted role colour with role-coloured border and text.
    Soft,
    /// Surface fill with a role-coloured border.
    Outline,
}

/// How nodes are drawn: a card with an icon tile (the house look) or the classic outline shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeStyle {
    #[default]
    Card,
    Shape,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Palette {
    pub primary: Rgb,
    pub secondary: Rgb,
    pub accent: Rgb,
    pub neutral: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub danger: Rgb,
}

impl Palette {
    pub fn get(&self, role: Role) -> Rgb {
        match role {
            Role::Primary => self.primary,
            Role::Secondary => self.secondary,
            Role::Accent => self.accent,
            Role::Neutral => self.neutral,
            Role::Success => self.success,
            Role::Warning => self.warning,
            Role::Danger => self.danger,
        }
    }

    pub fn set(&mut self, role: Role, c: Rgb) {
        match role {
            Role::Primary => self.primary = c,
            Role::Secondary => self.secondary = c,
            Role::Accent => self.accent = c,
            Role::Neutral => self.neutral = c,
            Role::Success => self.success = c,
            Role::Warning => self.warning = c,
            Role::Danger => self.danger = c,
        }
    }
}

/// Everything that makes a diagram look like one brand. Stored inside the document
/// and exportable on its own as a `.brand.json` so a kit can be shared across diagrams.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BrandKit {
    pub name: String,
    pub palette: Palette,
    pub background: Rgb,
    pub surface: Rgb,
    pub text: Rgb,
    pub muted_text: Rgb,
    /// CSS font stack used in SVG export and title text.
    pub font_family: String,
    pub heading_family: String,
    pub font_size: f32,
    pub corner_radius: f32,
    pub stroke_width: f32,
    pub edge_routing: EdgeRouting,
    pub node_fill: NodeFill,
    /// Card (icon tile, kind by icon) or Shape (outline per node shape).
    #[serde(default)]
    pub node_style: NodeStyle,
    pub shadow: bool,
    pub show_grid: bool,
    /// Optional inline SVG (full `<svg>` document) drawn top-left of exports.
    pub logo_svg: Option<String>,
}

impl Default for BrandKit {
    fn default() -> Self {
        BrandKit::rovelo()
    }
}

impl BrandKit {
    fn base(name: &str, palette: Palette) -> BrandKit {
        BrandKit {
            name: name.into(),
            palette,
            background: Rgb(0xfa, 0xfa, 0xf9),
            surface: Rgb(0xff, 0xff, 0xff),
            text: Rgb(0x1c, 0x1c, 0x1e),
            muted_text: Rgb(0x6b, 0x6b, 0x70),
            font_family: "Inter, -apple-system, 'Segoe UI', sans-serif".into(),
            heading_family: "Inter, -apple-system, 'Segoe UI', sans-serif".into(),
            font_size: 14.0,
            corner_radius: 10.0,
            stroke_width: 1.5,
            edge_routing: EdgeRouting::Orthogonal,
            node_fill: NodeFill::Soft,
            node_style: NodeStyle::Shape,
            shadow: false,
            show_grid: false,
            logo_svg: None,
        }
    }

    /// The house kit: Tailwind fuchsia with zinc neutrals.
    pub fn rovelo() -> BrandKit {
        let mut k = BrandKit::base(
            "Rovelo",
            Palette {
                primary: Rgb(0xc0, 0x26, 0xd3),
                secondary: Rgb(0x7c, 0x3a, 0xed),
                accent: Rgb(0xf5, 0x9e, 0x0b),
                neutral: Rgb(0x71, 0x71, 0x7a),
                success: Rgb(0x16, 0xa3, 0x4a),
                warning: Rgb(0xd9, 0x77, 0x06),
                danger: Rgb(0xdc, 0x26, 0x26),
            },
        );
        k.background = Rgb(0xfa, 0xfa, 0xfa);
        k.surface = Rgb(0xff, 0xff, 0xff);
        k.text = Rgb(0x18, 0x18, 0x1b);
        k.muted_text = Rgb(0x71, 0x71, 0x7a);
        k.corner_radius = 10.0;
        k.node_fill = NodeFill::Soft;
        k.edge_routing = EdgeRouting::Orthogonal;
        k.node_style = NodeStyle::Card;
        k
    }

    pub fn slate() -> BrandKit {
        BrandKit::base(
            "Slate",
            Palette {
                primary: Rgb(0x3b, 0x5b, 0xdb),
                secondary: Rgb(0x0c, 0x85, 0x99),
                accent: Rgb(0xf0, 0x8c, 0x00),
                neutral: Rgb(0x86, 0x8e, 0x96),
                success: Rgb(0x2f, 0x9e, 0x44),
                warning: Rgb(0xe6, 0x7e, 0x22),
                danger: Rgb(0xe0, 0x31, 0x31),
            },
        )
    }

    pub fn paper() -> BrandKit {
        let mut k = BrandKit::base(
            "Paper",
            Palette {
                primary: Rgb(0x2b, 0x2b, 0x2b),
                secondary: Rgb(0x6b, 0x5b, 0x45),
                accent: Rgb(0xc9, 0x4c, 0x2f),
                neutral: Rgb(0x8a, 0x84, 0x7a),
                success: Rgb(0x4a, 0x7c, 0x59),
                warning: Rgb(0xb8, 0x86, 0x0b),
                danger: Rgb(0xa6, 0x32, 0x32),
            },
        );
        k.background = Rgb(0xf6, 0xf1, 0xe7);
        k.surface = Rgb(0xfd, 0xfa, 0xf3);
        k.text = Rgb(0x2b, 0x2b, 0x2b);
        k.node_fill = NodeFill::Outline;
        k.corner_radius = 4.0;
        k.heading_family = "Georgia, 'Times New Roman', serif".into();
        k
    }

    pub fn midnight() -> BrandKit {
        let mut k = BrandKit::base(
            "Midnight",
            Palette {
                primary: Rgb(0x74, 0x8f, 0xfc),
                secondary: Rgb(0x3b, 0xc9, 0xdb),
                accent: Rgb(0xff, 0xd4, 0x3b),
                neutral: Rgb(0x86, 0x8e, 0x96),
                success: Rgb(0x69, 0xdb, 0x7c),
                warning: Rgb(0xff, 0xa9, 0x4d),
                danger: Rgb(0xff, 0x6b, 0x6b),
            },
        );
        k.background = Rgb(0x14, 0x15, 0x1a);
        k.surface = Rgb(0x1f, 0x21, 0x28);
        k.text = Rgb(0xec, 0xec, 0xef);
        k.muted_text = Rgb(0x9a, 0x9c, 0xa6);
        k.node_fill = NodeFill::Soft;
        k.edge_routing = EdgeRouting::Curved;
        k
    }

    pub fn presets() -> Vec<BrandKit> {
        vec![
            BrandKit::rovelo(),
            BrandKit::slate(),
            BrandKit::paper(),
            BrandKit::midnight(),
        ]
    }

    /// Fill, stroke and text colours for a node in `role` under this kit's fill mode.
    pub fn node_colors(&self, role: Role) -> NodeColors {
        let c = self.palette.get(role);
        match self.node_fill {
            NodeFill::Solid => NodeColors {
                fill: c,
                stroke: c,
                text: c.readable_on(),
            },
            NodeFill::Soft => {
                let fill = self.surface.mix(c, 0.16);
                let text = if fill.contrast(self.text) >= 4.5 {
                    self.text
                } else {
                    c.mix(self.text, 0.5)
                };
                NodeColors {
                    fill,
                    stroke: c,
                    text,
                }
            }
            NodeFill::Outline => NodeColors {
                fill: self.surface,
                stroke: c,
                text: self.text,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeColors {
    pub fill: Rgb,
    pub stroke: Rgb,
    pub text: Rgb,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        assert_eq!(Rgb::parse("#3b5bdb"), Some(Rgb(0x3b, 0x5b, 0xdb)));
        assert_eq!(Rgb::parse("f80"), Some(Rgb(0xff, 0x88, 0x00)));
        assert_eq!(Rgb(1, 2, 3).hex(), "#010203");
        assert_eq!(Rgb::parse("nope"), None);
    }

    #[test]
    fn readable_text_flips() {
        assert_eq!(Rgb(255, 255, 255).readable_on(), Rgb(17, 17, 17));
        assert_eq!(Rgb(0, 0, 0).readable_on(), Rgb(255, 255, 255));
    }

    #[test]
    fn every_preset_text_has_contrast() {
        for k in BrandKit::presets() {
            assert!(k.text.contrast(k.background) >= 7.0, "{}", k.name);
            for r in Role::ALL {
                let n = k.node_colors(r);
                assert!(n.text.contrast(n.fill) >= 4.5, "{} {:?}", k.name, r);
            }
        }
    }
}
