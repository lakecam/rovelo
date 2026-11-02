//! PNG export: rasterise the SVG string from `rovelo_core::svg::to_svg`.
use resvg::{tiny_skia, usvg};

/// Render `svg` at `scale`x. Errors are human-readable.
pub fn svg_to_png(svg: &str, scale: f32) -> Result<Vec<u8>, String> {
    let mut opt = usvg::Options::default();
    {
        let db = opt.fontdb_mut();
        db.load_system_fonts();
        for fam in ["Helvetica Neue", "Helvetica", "Arial"] {
            if db.faces().any(|f| f.families.iter().any(|(n, _)| n == fam)) {
                db.set_sans_serif_family(fam);
                break;
            }
        }
        for fam in ["Georgia", "Times New Roman", "Times"] {
            if db.faces().any(|f| f.families.iter().any(|(n, _)| n == fam)) {
                db.set_serif_family(fam);
                break;
            }
        }
    }
    let tree =
        usvg::Tree::from_str(svg, &opt).map_err(|e| format!("could not read the SVG: {e}"))?;
    let size = tree.size();
    let scale = scale.clamp(0.25, 8.0);
    let (w, h) = (
        (size.width() * scale).ceil(),
        (size.height() * scale).ceil(),
    );
    if !(w >= 1.0 && h >= 1.0) || w > 16_000.0 || h > 16_000.0 {
        return Err(format!(
            "diagram is {w}x{h}px, which is outside the exportable range"
        ));
    }
    let mut pixmap =
        tiny_skia::Pixmap::new(w as u32, h as u32).ok_or("could not allocate the image")?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|e| e.to_string())
}

/// Check that `svg` parses, for logo input.
pub fn validate_svg(svg: &str) -> Result<(), String> {
    if !svg.contains("<svg") {
        return Err("that does not look like an SVG (no <svg> element)".into());
    }
    usvg::Tree::from_str(svg, &usvg::Options::default())
        .map(|_| ())
        .map_err(|e| format!("could not read the SVG: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#3b5bdb"/></svg>"##;

    #[test]
    fn renders_png_at_2x() {
        let png = svg_to_png(SVG, 2.0).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        // IHDR width/height are big-endian u32 at bytes 16..24
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 80);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 40);
    }

    #[test]
    fn validates_logo_svg() {
        assert!(validate_svg(SVG).is_ok());
        assert!(validate_svg("hello").is_err());
        assert!(validate_svg("<svg").is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(svg_to_png("not svg", 2.0).is_err());
        assert!(svg_to_png("", 2.0).is_err());
    }
}
