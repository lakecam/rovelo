//! Native file dialogs and disk I/O. Kept apart so a web build can swap it out.
use rovelo_core::brand::BrandKit;
use rovelo_core::model::Diagram;
use std::path::{Path, PathBuf};

const DIAGRAM_EXT: &str = ".rovelo.json";
const BRAND_EXT: &str = ".brand.json";

pub fn pick_open_diagram() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Rovelo diagram", &["json"])
        .set_title("Open diagram")
        .pick_file()
}

pub fn pick_save_diagram(suggested: &str) -> Option<PathBuf> {
    let p = rfd::FileDialog::new()
        .add_filter("Rovelo diagram", &["json"])
        .set_file_name(format!("{}{DIAGRAM_EXT}", slug(suggested)))
        .set_title("Save diagram")
        .save_file()?;
    Some(with_suffix(p, DIAGRAM_EXT, &["json"]))
}

pub fn pick_save_export(suggested: &str, ext: &str, label: &str) -> Option<PathBuf> {
    let p = rfd::FileDialog::new()
        .add_filter(label, &[ext])
        .set_file_name(format!("{}.{ext}", slug(suggested)))
        .set_title(format!("Export {label}"))
        .save_file()?;
    Some(with_suffix(p, &format!(".{ext}"), &[ext]))
}

pub fn pick_open_brand() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("brand kit", &["json"])
        .set_title("Load brand kit")
        .pick_file()
}

pub fn pick_save_brand(name: &str) -> Option<PathBuf> {
    let p = rfd::FileDialog::new()
        .add_filter("brand kit", &["json"])
        .set_file_name(format!("{}{BRAND_EXT}", slug(name)))
        .set_title("Save brand kit")
        .save_file()?;
    Some(with_suffix(p, BRAND_EXT, &["json"]))
}

pub fn pick_open_svg() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("SVG image", &["svg"])
        .set_title("Choose a logo")
        .pick_file()
}

pub fn read_diagram(path: &Path) -> Result<Diagram, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    Diagram::from_json(&text)
}

pub fn write_diagram(path: &Path, d: &Diagram) -> Result<(), String> {
    write_text(path, &d.to_json())
}

pub fn read_brand(path: &Path) -> Result<BrandKit, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn write_brand(path: &Path, b: &BrandKit) -> Result<(), String> {
    let text = serde_json::to_string_pretty(b).map_err(|e| e.to_string())?;
    write_text(path, &text)
}

pub fn write_text(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

pub fn read_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

/// Document name for the title bar: file name without the double extension.
pub fn doc_name(path: Option<&Path>, title: &str) -> String {
    match path.and_then(|p| p.file_name()).and_then(|n| n.to_str()) {
        Some(n) => n
            .strip_suffix(DIAGRAM_EXT)
            .or_else(|| n.strip_suffix(".json"))
            .unwrap_or(n)
            .to_string(),
        None if title.trim().is_empty() => "Untitled".into(),
        None => title.trim().to_string(),
    }
}

fn slug(s: &str) -> String {
    let t: String = s
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let t = t.trim_matches('-').to_string();
    if t.is_empty() { "diagram".into() } else { t }
}

/// Make sure the saved name carries `suffix` (e.g. `.rovelo.json`) unless it already ends in it.
fn with_suffix(p: PathBuf, suffix: &str, plain_exts: &[&str]) -> PathBuf {
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    if name.ends_with(suffix) {
        return p;
    }
    let stem = match p.extension().and_then(|e| e.to_str()) {
        Some(e) if plain_exts.contains(&e) => name[..name.len() - e.len() - 1].to_string(),
        _ => name,
    };
    p.with_file_name(format!("{stem}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffixes_are_applied_once() {
        assert_eq!(
            with_suffix("/a/b/flow".into(), ".rovelo.json", &["json"]),
            PathBuf::from("/a/b/flow.rovelo.json")
        );
        assert_eq!(
            with_suffix("/a/b/flow.json".into(), ".rovelo.json", &["json"]),
            PathBuf::from("/a/b/flow.rovelo.json")
        );
        assert_eq!(
            with_suffix("/a/b/flow.rovelo.json".into(), ".rovelo.json", &["json"]),
            PathBuf::from("/a/b/flow.rovelo.json")
        );
        assert_eq!(
            with_suffix("/a/b/x.svg".into(), ".svg", &["svg"]),
            PathBuf::from("/a/b/x.svg")
        );
    }

    #[test]
    fn doc_names() {
        assert_eq!(
            doc_name(Some(Path::new("/x/My flow.rovelo.json")), "t"),
            "My flow"
        );
        assert_eq!(doc_name(None, "  "), "Untitled");
        assert_eq!(doc_name(None, "Plan"), "Plan");
        assert_eq!(slug("Sign-up flow!"), "Sign-up-flow");
        assert_eq!(slug("???"), "diagram");
    }

    #[test]
    fn diagram_round_trip_on_disk() {
        let dir = std::env::temp_dir().join(format!("rovelo-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.rovelo.json");
        let mut d = Diagram::default();
        d.ensure_node("a", "A");
        write_diagram(&p, &d).unwrap();
        assert_eq!(read_diagram(&p).unwrap(), d);
        let b = dir.join("k.brand.json");
        write_brand(&b, &BrandKit::paper()).unwrap();
        assert_eq!(read_brand(&b).unwrap(), BrandKit::paper());
        assert!(read_diagram(&dir.join("missing.json")).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
