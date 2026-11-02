//! Text form of a diagram: the rovelo language, plus Mermaid flowchart import.
//! The language is described in `GRAMMAR.md` next to this crate's manifest.
use rovelo_core::model::Diagram;

mod lexer;
mod mermaid;
mod parse;
mod serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// 1-based source line.
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct Parsed {
    pub diagram: Diagram,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse rovelo text. Forgiving: any input yields the best diagram it can, plus diagnostics.
/// Node positions are never in the text; callers run layout afterwards.
pub fn parse(src: &str) -> Parsed {
    parse::parse(src)
}

/// Canonical rovelo text for a diagram. `parse(serialize(d))` reproduces `d` apart from positions
/// and edge ids (edges are renumbered `e1`, `e2`, ... in order).
pub fn serialize(d: &Diagram) -> String {
    serialize::serialize(d)
}

/// Convert a Mermaid `flowchart`/`graph` source into a diagram, with warnings for what is dropped.
pub fn from_mermaid(src: &str) -> Parsed {
    mermaid::from_mermaid(src)
}
