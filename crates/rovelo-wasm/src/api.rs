//! Pure JSON-in / JSON-out wrappers over the Rust core. Everything here runs natively so it can be
//! tested with `cargo test`; `lib.rs` only adapts these to wasm-bindgen.
use rovelo_core::brand::{BrandKit, Role};
use rovelo_core::geometry::{self, EdgePath};
use rovelo_core::layout;
use rovelo_core::model::{Diagram, Point, Rect, Size};
use rovelo_core::svg;
use rovelo_dsl::{Diagnostic, Severity};
use serde::Serialize;

pub type Res = Result<String, String>;

fn read(json: &str) -> Result<Diagram, String> {
    Diagram::from_json(json)
}

fn out<T: Serialize>(v: &T) -> Res {
    serde_json::to_string(v).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct Diag {
    line: usize,
    severity: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ParsedOut {
    diagram: Diagram,
    diagnostics: Vec<Diag>,
}

fn parsed(p: rovelo_dsl::Parsed) -> Res {
    let sev = |s: Severity| match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };
    out(&ParsedOut {
        diagram: p.diagram,
        diagnostics: p
            .diagnostics
            .into_iter()
            .map(
                |Diagnostic {
                     line,
                     severity,
                     message,
                 }| Diag {
                    line,
                    severity: sev(severity),
                    message,
                },
            )
            .collect(),
    })
}

/// Rovelo text to `{diagram, diagnostics}`. Positions are unset; call `layout_unplaced`.
pub fn parse(text: &str) -> Res {
    parsed(rovelo_dsl::parse(text))
}

pub fn from_mermaid(text: &str) -> Res {
    parsed(rovelo_dsl::from_mermaid(text))
}

pub fn serialize(diagram: &str) -> Res {
    Ok(rovelo_dsl::serialize(&read(diagram)?))
}

/// Normalise a diagram: drop dangling edges and links.
pub fn repair(diagram: &str) -> Res {
    out(&read(diagram)?)
}

pub fn auto_layout(diagram: &str) -> Res {
    let mut d = read(diagram)?;
    layout::auto_layout(&mut d);
    out(&d)
}

pub fn layout_unplaced(diagram: &str) -> Res {
    let mut d = read(diagram)?;
    layout::layout_unplaced(&mut d);
    out(&d)
}

pub fn to_svg(diagram: &str) -> Res {
    Ok(svg::to_svg(&read(diagram)?))
}

#[derive(Serialize)]
struct EdgeGeo {
    points: Vec<Point>,
    label_at: Point,
    end_dir: Point,
    start_dir: Point,
}

impl From<EdgePath> for EdgeGeo {
    fn from(p: EdgePath) -> Self {
        EdgeGeo {
            points: p.points,
            label_at: p.label_at,
            end_dir: p.end_dir,
            start_dir: p.start_dir,
        }
    }
}

pub fn node_rect(diagram: &str, id: &str) -> Res {
    let d = read(diagram)?;
    let n = d.node(id).ok_or_else(|| format!("no node {id}"))?;
    out(&geometry::node_rect(&d, n))
}

pub fn node_size(diagram: &str, id: &str) -> Res {
    let d = read(diagram)?;
    let n = d.node(id).ok_or_else(|| format!("no node {id}"))?;
    out::<Size>(&geometry::node_size(&d, n))
}

pub fn edge_path(diagram: &str, id: &str) -> Res {
    let d = read(diagram)?;
    let e = d
        .edges
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("no edge {id}"))?;
    out(&EdgeGeo::from(geometry::edge_path(&d, e)))
}

pub fn group_rect(diagram: &str, id: &str) -> Res {
    let d = read(diagram)?;
    out(&geometry::group_rect(&d, id))
}

pub fn diagram_bounds(diagram: &str, margin: f32) -> Res {
    out(&geometry::diagram_bounds(&read(diagram)?, margin))
}

// ---- scene: everything the canvas needs in one call --------------------------------------

#[derive(Serialize)]
struct NodeScene {
    id: String,
    rect: Rect,
    svg: String,
}

#[derive(Serialize)]
struct EdgeScene {
    id: String,
    #[serde(flatten)]
    geo: EdgeGeo,
    label_rect: Option<Rect>,
    svg: String,
    label_svg: Option<String>,
}

#[derive(Serialize)]
struct GroupScene {
    id: String,
    rect: Rect,
    svg: String,
}

#[derive(Serialize)]
struct Badge {
    id: String,
    svg: String,
}

#[derive(Serialize)]
struct Scene {
    defs: String,
    bounds: Rect,
    nodes: Vec<NodeScene>,
    edges: Vec<EdgeScene>,
    groups: Vec<GroupScene>,
    badges: Vec<Badge>,
}

struct Frag {
    kind: String,
    id: String,
    inner: String,
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Split the exporter's output into its top-level `<g data-KIND="ID">` groups.
fn split_groups(svg_text: &str) -> Vec<Frag> {
    const KINDS: [&str; 5] = ["node", "edge", "edge-label", "group", "badge"];
    let end = svg_text.rfind("</svg>").unwrap_or(svg_text.len());
    let body = &svg_text[..end];
    let starts: Vec<usize> = body
        .match_indices("<g data-")
        .map(|(i, _)| i)
        .filter(|&i| {
            let rest = &body[i + 8..];
            KINDS.iter().any(|k| rest.starts_with(&format!("{k}=\"")))
        })
        .collect();
    let mut frags = Vec::new();
    for (n, &s) in starts.iter().enumerate() {
        let e = starts.get(n + 1).copied().unwrap_or(body.len());
        let seg = body[s..e].trim_end();
        let Some(open_end) = seg.find('>') else {
            continue;
        };
        let open = &seg[8..open_end];
        let (Some(eq), Some(q1)) = (open.find("=\""), open.find('"')) else {
            continue;
        };
        let Some(q2) = open.rfind('"') else { continue };
        let kind = open[..eq].to_string();
        let id = unescape(&open[q1 + 1..q2]);
        let inner = seg[open_end + 1..]
            .strip_suffix("</g>")
            .unwrap_or(&seg[open_end + 1..])
            .to_string();
        frags.push(Frag { kind, id, inner });
    }
    frags
}

/// Geometry plus the exporter's own markup for each part, so the interactive canvas draws exactly
/// what an SVG export draws. Unplaced nodes are laid out on a copy first.
pub fn scene(diagram: &str) -> Res {
    let mut d = read(diagram)?;
    layout::layout_unplaced(&mut d);
    let text = svg::to_svg(&d);
    let frags = split_groups(&text);
    let find = |kind: &str, id: &str| {
        frags
            .iter()
            .find(|f| f.kind == kind && f.id == id)
            .map(|f| f.inner.clone())
    };
    let defs = text
        .split_once("<defs>")
        .and_then(|(_, r)| r.split_once("</defs>"))
        .map(|(a, _)| a.to_string())
        .unwrap_or_default();
    let nodes = d
        .nodes
        .iter()
        .map(|n| NodeScene {
            id: n.id.clone(),
            rect: geometry::node_rect(&d, n),
            svg: find("node", &n.id).unwrap_or_default(),
        })
        .collect();
    let edges = d
        .edges
        .iter()
        .map(|e| {
            let p = geometry::edge_path(&d, e);
            // The same chip (cards) or pill (shapes) the exporter draws.
            let label_rect = geometry::edge_label_rect(&d, e, &p);
            EdgeScene {
                id: e.id.clone(),
                geo: p.into(),
                label_rect,
                svg: find("edge", &e.id).unwrap_or_default(),
                label_svg: find("edge-label", &e.id),
            }
        })
        .collect();
    let groups = d
        .groups
        .iter()
        .filter_map(|g| {
            Some(GroupScene {
                id: g.id.clone(),
                rect: geometry::group_rect(&d, &g.id)?,
                svg: find("group", &g.id).unwrap_or_default(),
            })
        })
        .collect();
    let badges = frags
        .iter()
        .filter(|f| f.kind == "badge")
        .map(|f| Badge {
            id: f.id.clone(),
            svg: f.inner.clone(),
        })
        .collect();
    out(&Scene {
        defs,
        bounds: geometry::diagram_bounds(&d, 0.0),
        nodes,
        edges,
        groups,
        badges,
    })
}

// ---- brand -------------------------------------------------------------------------------

pub fn brand_presets() -> Res {
    out(&BrandKit::presets())
}

/// Check a `.brand.json` and return it normalised.
pub fn validate_brand(json: &str) -> Res {
    let k: BrandKit = serde_json::from_str(json).map_err(|e| e.to_string())?;
    out(&k)
}

#[derive(Serialize)]
struct Colors {
    fill: String,
    stroke: String,
    text: String,
}

/// Fill, stroke and text colours a node of `role` gets under a brand kit.
pub fn node_colors(brand: &str, role: &str) -> Res {
    let k: BrandKit = serde_json::from_str(brand).map_err(|e| e.to_string())?;
    let role = Role::from_name(role).ok_or_else(|| format!("unknown role {role}"))?;
    let c = k.node_colors(role);
    out(&Colors {
        fill: c.fill.hex(),
        stroke: c.stroke.hex(),
        text: c.text.hex(),
    })
}

pub fn icon_names() -> Res {
    out(&svg::ICON_NAMES)
}

// ---- constructors ------------------------------------------------------------------------

pub fn default_diagram() -> Res {
    out(&Diagram::default())
}

/// A starter diagram. `brand` may be empty to use the default kit.
pub fn template(name: &str, today: &str, brand: &str) -> Res {
    let kit = if brand.trim().is_empty() {
        BrandKit::default()
    } else {
        serde_json::from_str(brand).map_err(|e| e.to_string())?
    };
    out(&crate::templates::build(name, today, &kit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn json(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }

    fn sample() -> String {
        template("taskflow", "2026-11-02", "").unwrap()
    }

    #[test]
    fn parse_serialize_round_trip() {
        let p = json(&parse("taskflow right\ntitle: T\nA -> B -> C").unwrap());
        assert_eq!(p["diagram"]["nodes"].as_array().unwrap().len(), 3);
        assert_eq!(p["diagram"]["direction"], "right");
        assert!(p["diagnostics"].as_array().unwrap().is_empty());
        let text = serialize(&p["diagram"].to_string()).unwrap();
        let again = json(&parse(&text).unwrap());
        assert_eq!(again["diagram"]["edges"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn diagnostics_have_line_and_severity() {
        let p = json(&parse("A -> B\nA [shape: blob]").unwrap());
        let d = &p["diagnostics"][0];
        assert_eq!(d["line"], 2);
        assert_eq!(d["severity"], "warning");
    }

    #[test]
    fn mermaid_import() {
        let p = json(&from_mermaid("flowchart LR\nA[Start] --> B{Ok?}\nB -->|yes| C").unwrap());
        assert_eq!(p["diagram"]["nodes"].as_array().unwrap().len(), 3);
        assert_eq!(p["diagram"]["direction"], "right");
    }

    #[test]
    fn layout_places_nodes() {
        let p = json(&parse("A -> B").unwrap());
        assert!(p["diagram"]["nodes"][0]["pos"].is_null());
        let laid = json(&layout_unplaced(&p["diagram"].to_string()).unwrap());
        assert!(laid["nodes"][0]["pos"]["x"].is_number());
        let re = json(&auto_layout(&laid.to_string()).unwrap());
        assert!(
            re["nodes"][1]["pos"]["y"].as_f64().unwrap()
                > re["nodes"][0]["pos"]["y"].as_f64().unwrap()
        );
    }

    #[test]
    fn svg_export_is_svg() {
        let s = to_svg(&sample()).unwrap();
        assert!(s.starts_with("<svg") && s.ends_with("</svg>"));
        assert!(svg::is_well_formed(&s));
    }

    #[test]
    fn scene_covers_every_part() {
        let d = sample();
        let v = json(&scene(&d).unwrap());
        let src = json(&d);
        assert_eq!(
            v["nodes"].as_array().unwrap().len(),
            src["nodes"].as_array().unwrap().len()
        );
        assert_eq!(
            v["edges"].as_array().unwrap().len(),
            src["edges"].as_array().unwrap().len()
        );
        assert!(
            v["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .all(|n| n["svg"].as_str().unwrap().len() > 10
                    && n["rect"]["w"].as_f64().unwrap() > 0.0)
        );
        assert!(
            v["edges"]
                .as_array()
                .unwrap()
                .iter()
                .all(|e| e["points"].as_array().unwrap().len() >= 2
                    && !e["svg"].as_str().unwrap().is_empty())
        );
        assert!(
            v["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["label_svg"].is_string() && e["label_rect"].is_object())
        );
        assert_eq!(v["groups"].as_array().unwrap().len(), 1);
        // Card style: no floating badges; to-do progress is the card's subtitle.
        assert!(v["badges"].as_array().unwrap().is_empty());
        assert!(
            v["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["svg"].as_str().unwrap().contains("to-do"))
        );
        // Edge label chips are 20 tall.
        assert!(
            v["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|e| e["label_rect"]["h"].as_f64())
                .all(|h| (h - 20.0).abs() < 0.01)
        );
        assert!(v["bounds"]["w"].as_f64().unwrap() > 100.0);
    }

    #[test]
    fn scene_handles_odd_ids_and_unplaced() {
        let p = json(&parse("a&b = \"x <y>\"\na&b -> c").unwrap());
        let v = json(&scene(&p["diagram"].to_string()).unwrap());
        let n = &v["nodes"][0];
        assert_eq!(n["id"], "a&b");
        assert!(n["svg"].as_str().unwrap().contains("&lt;y&gt;"));
    }

    #[test]
    fn geometry_lookups() {
        let d = sample();
        let id = json(&d)["nodes"][0]["id"].as_str().unwrap().to_string();
        assert!(json(&node_rect(&d, &id).unwrap())["w"].as_f64().unwrap() > 0.0);
        assert!(node_rect(&d, "nope").is_err());
        let eid = json(&d)["edges"][0]["id"].as_str().unwrap().to_string();
        assert!(
            json(&edge_path(&d, &eid).unwrap())["points"]
                .as_array()
                .unwrap()
                .len()
                >= 2
        );
        assert!(json(&group_rect(&d, "g_qa").unwrap())["w"].is_number());
        assert!(json(&group_rect(&d, "zz").unwrap()).is_null());
        assert!(
            json(&diagram_bounds(&d, 24.0).unwrap())["h"]
                .as_f64()
                .unwrap()
                > 200.0
        );
    }

    #[test]
    fn brand_helpers() {
        let presets = json(&brand_presets().unwrap());
        assert!(presets.as_array().unwrap().len() >= 3);
        let one = presets[0].to_string();
        assert_eq!(json(&validate_brand(&one).unwrap()), presets[0]);
        assert!(validate_brand("{\"name\":1}").is_err());
        let c = json(&node_colors(&one, "danger").unwrap());
        assert!(c["fill"].as_str().unwrap().starts_with('#'));
        assert!(node_colors(&one, "nope").is_err());
    }

    #[test]
    fn bad_input_is_an_error_not_a_panic() {
        assert!(serialize("{").is_err());
        assert!(scene("[]").is_err());
        assert!(template("blank", "2026-11-02", "{").is_err());
    }

    #[test]
    fn constructors() {
        assert_eq!(
            json(&default_diagram().unwrap())["nodes"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        for n in crate::templates::NAMES {
            assert!(template(n, "2026-11-02", "").is_ok());
        }
        assert!(json(&icon_names().unwrap()).as_array().unwrap().len() >= 12);
    }
}
