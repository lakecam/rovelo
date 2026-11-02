//! The rovelo text parser: forgiving, line-oriented, never panics.
use std::collections::{HashMap, HashSet};

use rovelo_core::brand::{BrandKit, EdgeRouting, NodeFill, NodeStyle, Rgb, Role};
use rovelo_core::model::{
    Arrow, Diagram, DiagramKind, Direction, Edge, Group, LineStyle, Node, Priority, Shape, Size,
    Task,
};

use crate::lexer::{Kind, Tok, lex, strip_comment};
use crate::{Diagnostic, Parsed, Severity};

pub const MAX_LINES: usize = 20_000;
/// Cap on edges one fan-out statement may expand to.
const MAX_FANOUT: usize = 50_000;

/// Id-safe form of a label: lowercase, runs of non-alphanumerics become one `-`.
pub fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

pub fn parse(src: &str) -> Parsed {
    let mut p = Parser::new();
    p.run(src);
    Parsed {
        diagram: p.d,
        diagnostics: p.diags,
    }
}

struct Attr<'a> {
    key: String,
    toks: &'a [Tok],
    flag: bool,
}

struct Parser {
    d: Diagram,
    diags: Vec<Diagnostic>,
    line: usize,
    node_idx: HashMap<String, usize>,
    label_idx: HashMap<String, Vec<usize>>,
    group_idx: HashMap<String, usize>,
    task_ids: HashSet<String>,
    declared: HashSet<String>,
    /// Open `{` blocks: the group each one fills, or `None` for a stray brace. Paired with the opening line.
    stack: Vec<(Option<usize>, usize)>,
    edge_n: usize,
}

fn text_of(t: &Tok, src: &str) -> String {
    match &t.kind {
        Kind::Str(s) => s.clone(),
        _ => src[t.start..t.end].to_string(),
    }
}

/// A name, label or value from tokens: one quoted string decodes; anything else is the raw source text.
fn joined(toks: &[Tok], src: &str) -> String {
    match toks {
        [] => String::new(),
        [t] => text_of(t, src),
        _ => src[toks[0].start..toks[toks.len() - 1].end].to_string(),
    }
}

/// Split off a trailing `[ ... ]` block, matching brackets from the right.
fn trailing_block(toks: &[Tok]) -> (&[Tok], Option<&[Tok]>, bool) {
    if toks.last().map(|t| &t.kind) != Some(&Kind::RBracket) {
        return (toks, None, true);
    }
    let mut depth = 0usize;
    for k in (0..toks.len()).rev() {
        match toks[k].kind {
            Kind::RBracket => depth += 1,
            Kind::LBracket => {
                depth -= 1;
                if depth == 0 {
                    return (&toks[..k], Some(&toks[k + 1..toks.len() - 1]), true);
                }
            }
            _ => {}
        }
    }
    (&toks[..toks.len() - 1], None, false)
}

fn split_commas(toks: &[Tok]) -> Vec<&[Tok]> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut from = 0;
    for (i, t) in toks.iter().enumerate() {
        match t.kind {
            Kind::LBracket => depth += 1,
            Kind::RBracket => depth = depth.saturating_sub(1),
            Kind::Comma if depth == 0 => {
                out.push(&toks[from..i]);
                from = i + 1;
            }
            _ => {}
        }
    }
    out.push(&toks[from..]);
    out
}

fn bool_of(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "on" | "true" | "yes" | "1" => Some(true),
        "off" | "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

fn color_of(s: &str) -> Option<Rgb> {
    let t = s.trim();
    let t = t.strip_prefix('#').unwrap_or(t);
    if (t.len() == 3 || t.len() == 6) && t.bytes().all(|b| b.is_ascii_hexdigit()) {
        Rgb::parse(t)
    } else {
        None
    }
}

fn number_of(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok().filter(|v| v.is_finite())
}

fn word_cases(s: &str) -> [String; 3] {
    [s.to_string(), s.to_lowercase(), s.to_uppercase()]
}

fn is_key_line(toks: &[Tok], src: &str) -> bool {
    if toks.len() < 2 || toks[0].kind != Kind::Word || toks[1].kind != Kind::Colon {
        return false;
    }
    let k = src[toks[0].start..toks[0].end].to_ascii_lowercase();
    k == "title" || k == "brand" || k.starts_with("brand.")
}

impl Parser {
    fn new() -> Parser {
        Parser {
            d: Diagram::default(),
            diags: vec![],
            line: 0,
            node_idx: HashMap::new(),
            label_idx: HashMap::new(),
            group_idx: HashMap::new(),
            task_ids: HashSet::new(),
            declared: HashSet::new(),
            stack: vec![],
            edge_n: 0,
        }
    }

    fn diag(&mut self, severity: Severity, message: impl Into<String>) {
        self.diags.push(Diagnostic {
            line: self.line,
            severity,
            message: message.into(),
        });
    }

    fn warn(&mut self, m: impl Into<String>) {
        self.diag(Severity::Warning, m);
    }

    fn run(&mut self, src: &str) {
        let src = src.strip_prefix('\u{feff}').unwrap_or(src);
        for (i, raw) in src.lines().enumerate() {
            if i >= MAX_LINES {
                self.line = MAX_LINES + 1;
                self.diag(
                    Severity::Error,
                    format!("input is longer than {MAX_LINES} lines; the rest was ignored"),
                );
                break;
            }
            self.line = i + 1;
            self.line_of(raw);
        }
        let open: Vec<usize> = self
            .stack
            .iter()
            .filter(|(g, _)| g.is_some())
            .map(|(_, l)| *l)
            .collect();
        for l in open {
            self.line = l;
            self.warn("group block is never closed with `}`");
        }
    }

    fn line_of(&mut self, raw: &str) {
        let s = strip_comment(raw).trim_end();
        let t = s.trim();
        if t.is_empty() {
            return;
        }
        if let Some(rest) = t.strip_prefix("%%") {
            let mut w = rest.split_whitespace();
            if w.next() == Some("rovelo") {
                match w.next() {
                    Some("1") => {}
                    Some(v) => self.warn(format!(
                        "rovelo version `{v}` is not known; reading it as version 1"
                    )),
                    None => {}
                }
            }
            return;
        }
        let lx = lex(s);
        if lx.unterminated {
            self.warn("string is missing its closing quote");
        }
        let toks = lx.toks;
        if is_key_line(&toks, s) {
            self.key_line(&toks, s);
            return;
        }
        let mut seg_start = 0;
        let mut depth = 0usize;
        for i in 0..=toks.len() {
            let kind = toks.get(i).map(|t| &t.kind);
            match kind {
                Some(Kind::LBracket) => depth += 1,
                Some(Kind::RBracket) => depth = depth.saturating_sub(1),
                _ => {}
            }
            let split =
                depth == 0 && matches!(kind, None | Some(Kind::Semi | Kind::LBrace | Kind::RBrace));
            if !split {
                continue;
            }
            let seg = &toks[seg_start..i];
            match kind {
                Some(Kind::LBrace) => self.open_block(seg, s),
                Some(Kind::RBrace) => {
                    self.segment(seg, s);
                    if self.stack.pop().is_none() {
                        self.warn("`}` closes nothing");
                    }
                }
                _ => self.segment(seg, s),
            }
            seg_start = i + 1;
        }
    }

    fn segment(&mut self, seg: &[Tok], src: &str) {
        let Some(first) = seg.first() else { return };
        if seg.len() >= 2
            && matches!(first.kind, Kind::Word | Kind::Str(_))
            && seg[1].kind == Kind::Eq
        {
            return self.declaration(seg, src);
        }
        if first.kind == Kind::Word {
            let w = src[first.start..first.end].to_ascii_lowercase();
            match w.as_str() {
                "todo" => return self.todo(seg, src),
                "group" if !seg.iter().any(|t| matches!(t.kind, Kind::Arrow(_))) => {
                    self.warn("`group` needs a `{ ... }` block");
                    self.group_header(seg, src);
                    return;
                }
                "flowchart" | "userflow" | "taskflow"
                    if seg.len() == 1 || (seg.len() == 2 && seg[1].kind == Kind::Word) =>
                {
                    return self.family(seg, src);
                }
                _ => {}
            }
            if is_key_line(seg, src) {
                return self.key_line(seg, src);
            }
        }
        self.chain(seg, src);
    }

    fn current_group(&self) -> Option<usize> {
        self.stack.iter().rev().find_map(|(g, _)| *g)
    }

    fn assign(&mut self, ni: usize) {
        let Some(gi) = self.current_group() else {
            return;
        };
        let gid = self.d.groups[gi].id.clone();
        let node = &mut self.d.nodes[ni];
        match &node.group {
            Some(g) if *g == gid => {}
            Some(g) => {
                let (nid, old) = (node.id.clone(), g.clone());
                node.group = Some(gid.clone());
                self.warn(format!(
                    "node `{nid}` moved from group `{old}` to `{gid}`; a node has one group"
                ));
            }
            None => node.group = Some(gid),
        }
    }

    // ---- nodes -----------------------------------------------------------------------------

    /// `base`, or `base-2`, `base-3`... until `taken` says no. An empty slug falls back to `fallback`.
    fn unique(base: &str, fallback: &str, taken: impl Fn(&str) -> bool) -> String {
        let base = if base.is_empty() { fallback } else { base };
        if !taken(base) {
            return base.to_string();
        }
        (2..)
            .map(|n| format!("{base}-{n}"))
            .find(|c| !taken(c))
            .unwrap_or_default()
    }

    fn create_node(&mut self, id: String, label: String) -> usize {
        let i = self.d.nodes.len();
        self.node_idx.insert(id.clone(), i);
        self.label_idx.entry(label.clone()).or_default().push(i);
        self.d.nodes.push(Node::new(id, label));
        i
    }

    fn set_label(&mut self, i: usize, label: String) {
        let old = std::mem::replace(&mut self.d.nodes[i].label, label.clone());
        if let Some(v) = self.label_idx.get_mut(&old) {
            v.retain(|x| *x != i);
        }
        let v = self.label_idx.entry(label).or_default();
        let at = v.partition_point(|x| *x < i);
        v.insert(at, i);
    }

    /// Endpoint resolution: an existing id, then an existing label, else a new node named by the text.
    fn resolve(&mut self, text: &str) -> usize {
        if let Some(&i) = self.node_idx.get(text) {
            return i;
        }
        if let Some(&i) = self.label_idx.get(text).and_then(|v| v.first()) {
            return i;
        }
        let id = Self::unique(&slug(text), "node", |c| self.node_idx.contains_key(c));
        self.create_node(id, text.to_string())
    }

    fn declaration(&mut self, seg: &[Tok], src: &str) {
        let id = text_of(&seg[0], src);
        let (head, attrs, ok) = trailing_block(&seg[2..]);
        if !ok {
            self.warn("`]` without a matching `[`");
        }
        let label = if head.is_empty() {
            id.clone()
        } else {
            joined(head, src)
        };
        let idx = match self.node_idx.get(&id).copied() {
            Some(i) => {
                if self.declared.contains(&id) {
                    self.warn(format!(
                        "`{id}` is declared twice; the later label and attributes win"
                    ));
                }
                self.set_label(i, label);
                i
            }
            None => self.create_node(id, label),
        };
        self.declared.insert(self.d.nodes[idx].id.clone());
        if let Some(a) = attrs {
            let attrs = self.attrs(a, src);
            self.node_attrs(idx, &attrs, src);
        }
        self.assign(idx);
    }

    fn node_attrs(&mut self, idx: usize, attrs: &[Attr], src: &str) {
        for a in attrs {
            let v = joined(a.toks, src);
            let lower = v.to_lowercase();
            match a.key.as_str() {
                "shape" if !a.flag => match Shape::from_name(&lower) {
                    Some(s) => self.d.nodes[idx].shape = s,
                    None => self.warn(format!("unknown shape `{v}`")),
                },
                "role" if !a.flag => match Role::from_name(&lower) {
                    Some(r) => self.d.nodes[idx].role = r,
                    None => self.warn(format!("unknown role `{v}`")),
                },
                "icon" if !a.flag && !a.toks.is_empty() => self.d.nodes[idx].icon = Some(v),
                "size" if !a.flag => {
                    let parsed = lower.split_once(['x', '*']).and_then(|(w, h)| {
                        Some(Size {
                            w: number_of(w)?,
                            h: number_of(h)?,
                        })
                    });
                    match parsed {
                        Some(s) => self.d.nodes[idx].size = Some(s),
                        None => self.warn(format!("size `{v}` should look like 120x60")),
                    }
                }
                k => self.warn(format!("unknown or incomplete node attribute `{k}`")),
            }
        }
    }

    // ---- attribute lists -------------------------------------------------------------------

    fn attrs<'a>(&mut self, toks: &'a [Tok], src: &str) -> Vec<Attr<'a>> {
        let mut out = Vec::new();
        for item in split_commas(toks) {
            let Some(first) = item.first() else { continue };
            if !matches!(first.kind, Kind::Word | Kind::Str(_)) {
                self.warn("malformed attribute; expected `name` or `name: value`");
                continue;
            }
            let key = text_of(first, src).to_lowercase().replace('-', "_");
            match item.get(1).map(|t| &t.kind) {
                None => out.push(Attr {
                    key,
                    toks: &item[1..],
                    flag: true,
                }),
                Some(Kind::Colon | Kind::Eq) => out.push(Attr {
                    key,
                    toks: &item[2..],
                    flag: false,
                }),
                Some(_) => self.warn(format!(
                    "malformed attribute `{key}`; expected `{key}: value`"
                )),
            }
        }
        out
    }

    // ---- statements ------------------------------------------------------------------------

    fn family(&mut self, seg: &[Tok], src: &str) {
        let k = src[seg[0].start..seg[0].end].to_ascii_lowercase();
        if let Some(kind) = DiagramKind::from_name(&k) {
            self.d.kind = kind;
        }
        if let Some(t) = seg.get(1) {
            let w = &src[t.start..t.end];
            match word_cases(w).iter().find_map(|c| Direction::from_name(c)) {
                Some(dir) => self.d.direction = dir,
                None => self.warn(format!(
                    "unknown direction `{w}`; use down, right, left or up"
                )),
            }
        }
    }

    fn key_line(&mut self, toks: &[Tok], src: &str) {
        let key = src[toks[0].start..toks[0].end].to_ascii_lowercase();
        let value = joined(&toks[2..], src);
        let value = value.as_str();
        let quoted = matches!(
            toks[2..],
            [Tok {
                kind: Kind::Str(_),
                ..
            }]
        );
        if key == "title" {
            self.d.title = value.to_string();
            return;
        }
        if key == "brand" {
            match BrandKit::presets()
                .into_iter()
                .find(|p| p.name.eq_ignore_ascii_case(value))
            {
                Some(p) => self.d.brand = p,
                None => self.warn(format!(
                    "unknown brand preset `{value}`; use rovelo, slate, paper or midnight"
                )),
            }
            return;
        }
        let k = key["brand.".len()..].replace('-', "_");
        self.brand_key(&k, value, quoted);
    }

    fn brand_key(&mut self, k: &str, v: &str, quoted: bool) {
        let bad = |s: &mut Parser, what: &str| s.warn(format!("brand.{k}: `{v}` is not {what}"));
        if let Some(role) = Role::from_name(k) {
            match color_of(v) {
                Some(c) => self.d.brand.palette.set(role, c),
                None => bad(self, "a colour like #3b5bdb"),
            }
            return;
        }
        match k {
            "name" => self.d.brand.name = v.to_string(),
            "background" | "surface" | "text" | "muted_text" | "muted" => match color_of(v) {
                Some(c) => {
                    let b = &mut self.d.brand;
                    match k {
                        "background" => b.background = c,
                        "surface" => b.surface = c,
                        "text" => b.text = c,
                        _ => b.muted_text = c,
                    }
                }
                None => bad(self, "a colour like #3b5bdb"),
            },
            "font" | "font_family" => self.d.brand.font_family = v.to_string(),
            "heading_font" | "heading_family" => self.d.brand.heading_family = v.to_string(),
            "radius" | "corner_radius" | "stroke" | "stroke_width" | "size" | "font_size" => {
                match number_of(v) {
                    Some(n) => {
                        let b = &mut self.d.brand;
                        match k {
                            "radius" | "corner_radius" => b.corner_radius = n,
                            "stroke" | "stroke_width" => b.stroke_width = n,
                            _ => b.font_size = n,
                        }
                    }
                    None => bad(self, "a number"),
                }
            }
            "routing" => match v.to_lowercase().as_str() {
                "straight" => self.d.brand.edge_routing = EdgeRouting::Straight,
                "orthogonal" => self.d.brand.edge_routing = EdgeRouting::Orthogonal,
                "curved" => self.d.brand.edge_routing = EdgeRouting::Curved,
                _ => bad(self, "straight, orthogonal or curved"),
            },
            "fill" => match v.to_lowercase().as_str() {
                "solid" => self.d.brand.node_fill = NodeFill::Solid,
                "soft" => self.d.brand.node_fill = NodeFill::Soft,
                "outline" => self.d.brand.node_fill = NodeFill::Outline,
                _ => bad(self, "solid, soft or outline"),
            },
            "nodes" | "node_style" => match v.to_lowercase().as_str() {
                "card" => self.d.brand.node_style = NodeStyle::Card,
                "shape" => self.d.brand.node_style = NodeStyle::Shape,
                _ => bad(self, "card or shape"),
            },
            "shadow" | "grid" => match bool_of(v) {
                Some(b) if k == "shadow" => self.d.brand.shadow = b,
                Some(b) => self.d.brand.show_grid = b,
                None => bad(self, "on or off"),
            },
            "logo" => {
                self.d.brand.logo_svg = if v.eq_ignore_ascii_case("none") && !quoted {
                    None
                } else {
                    Some(v.to_string())
                };
            }
            _ => self.warn(format!("unknown brand key `brand.{k}`")),
        }
    }
}

impl Parser {
    // ---- groups ----------------------------------------------------------------------------

    fn create_group(&mut self, id: String, label: String) -> usize {
        let gi = self.d.groups.len();
        self.group_idx.insert(id.clone(), gi);
        self.d.groups.push(Group {
            id,
            label,
            role: Role::Neutral,
        });
        gi
    }

    /// `group [id =] label [role: x]`; returns the group's index, creating it unless the explicit id exists.
    fn group_header(&mut self, seg: &[Tok], src: &str) -> usize {
        let (head, attrs, ok) = trailing_block(&seg[1..]);
        if !ok {
            self.warn("`]` without a matching `[`");
        }
        let explicit = head.len() >= 2
            && matches!(head[0].kind, Kind::Word | Kind::Str(_))
            && head[1].kind == Kind::Eq;
        let (id_text, label_toks) = if explicit {
            (Some(text_of(&head[0], src)), &head[2..])
        } else {
            (None, head)
        };
        let label = if label_toks.is_empty() {
            None
        } else {
            Some(joined(label_toks, src))
        };
        let gi = match id_text {
            Some(id) if self.group_idx.contains_key(&id) => {
                let gi = self.group_idx[&id];
                if let Some(l) = label {
                    self.d.groups[gi].label = l;
                }
                gi
            }
            Some(id) => {
                let label = label.unwrap_or_else(|| id.clone());
                self.create_group(id, label)
            }
            None => {
                let label = label.unwrap_or_else(|| "Group".to_string());
                let id = Self::unique(&slug(&label), "group", |c| self.group_idx.contains_key(c));
                self.create_group(id, label)
            }
        };
        if let Some(a) = attrs {
            for at in self.attrs(a, src) {
                let v = joined(at.toks, src);
                match at.key.as_str() {
                    "role" if !at.flag => match Role::from_name(&v.to_lowercase()) {
                        Some(r) => self.d.groups[gi].role = r,
                        None => self.warn(format!("unknown role `{v}`")),
                    },
                    k => self.warn(format!("unknown or incomplete group attribute `{k}`")),
                }
            }
        }
        gi
    }

    fn open_block(&mut self, seg: &[Tok], src: &str) {
        let is_group = seg.first().is_some_and(|t| {
            t.kind == Kind::Word && src[t.start..t.end].eq_ignore_ascii_case("group")
        });
        if is_group {
            if self.current_group().is_some() {
                self.warn(
                    "groups do not nest; the inner group is kept as a separate top-level group",
                );
            }
            let gi = self.group_header(seg, src);
            self.stack.push((Some(gi), self.line));
        } else {
            self.warn("`{` only opens a block after `group ...`");
            self.segment(seg, src);
            self.stack.push((None, self.line));
        }
    }

    // ---- tasks -----------------------------------------------------------------------------

    fn auto_task_id(&self) -> String {
        (self.d.tasks.len() + 1..)
            .map(|n| format!("t{n}"))
            .find(|c| !self.task_ids.contains(c))
            .unwrap_or_default()
    }

    fn todo(&mut self, seg: &[Tok], src: &str) {
        let (head, attrs, ok) = trailing_block(&seg[1..]);
        if !ok {
            self.warn("`]` without a matching `[`");
        }
        let colon = head.iter().position(|t| t.kind == Kind::Colon);
        let (ref_toks, title_toks) = match colon {
            Some(c) => (&head[..c], &head[c + 1..]),
            None => (&head[..0], head),
        };
        if title_toks.is_empty() {
            self.warn("todo has no title");
        }
        let title = joined(title_toks, src);
        let node = if ref_toks.is_empty() {
            None
        } else {
            let t = joined(ref_toks, src);
            if !self.node_idx.contains_key(&t) && !self.label_idx.contains_key(&t) {
                self.warn(format!(
                    "todo refers to `{t}`, which is not a node yet; created it"
                ));
            }
            let i = self.resolve(&t);
            Some(self.d.nodes[i].id.clone())
        };
        let attrs = attrs.map(|a| self.attrs(a, src)).unwrap_or_default();
        let explicit = attrs
            .iter()
            .find(|a| a.key == "id" && !a.flag)
            .map(|a| joined(a.toks, src));
        let id = match explicit {
            Some(e) if !e.is_empty() && !self.task_ids.contains(&e) => e,
            Some(e) => {
                self.warn(format!(
                    "task id `{e}` is empty or already used; assigned a new one"
                ));
                self.auto_task_id()
            }
            None => self.auto_task_id(),
        };
        self.task_ids.insert(id.clone());
        let mut t = Task::new(id, title);
        t.node = node;
        for a in &attrs {
            let v = joined(a.toks, src);
            match a.key.as_str() {
                "id" => {}
                "done" => {
                    if a.flag {
                        t.done = true;
                    } else {
                        match bool_of(&v) {
                            Some(b) => t.done = b,
                            None => self.warn(format!("done: `{v}` is not true or false")),
                        }
                    }
                }
                "due" if !a.flag && !a.toks.is_empty() => {
                    if !looks_iso(&v) {
                        self.warn(format!("due date `{v}` is not YYYY-MM-DD"));
                    }
                    t.due = Some(v);
                }
                "priority" if !a.flag => match Priority::from_name(&v.to_lowercase()) {
                    Some(p) => t.priority = p,
                    None => self.warn(format!("unknown priority `{v}`; use low, normal or high")),
                },
                "tags" if !a.flag => {
                    t.tags = a
                        .toks
                        .iter()
                        .filter(|k| matches!(k.kind, Kind::Word | Kind::Str(_)))
                        .map(|k| text_of(k, src))
                        .collect();
                }
                "notes" if !a.flag => t.notes = v,
                k => self.warn(format!("unknown or incomplete todo attribute `{k}`")),
            }
        }
        self.d.tasks.push(t);
    }

    // ---- edges -----------------------------------------------------------------------------

    fn chain(&mut self, seg: &[Tok], src: &str) {
        let (head, attrs, ok) = trailing_block(seg);
        if !ok {
            self.warn("`]` without a matching `[`");
        }
        if head.is_empty() {
            if attrs.is_some() {
                self.warn("attributes need a node or edge in front of them");
            }
            return;
        }
        let attrs = attrs.map(|a| self.attrs(a, src)).unwrap_or_default();
        let Some(fa) = head.iter().position(|t| matches!(t.kind, Kind::Arrow(_))) else {
            if head.iter().any(|t| t.kind == Kind::Colon) {
                self.warn("not a statement rovelo understands (no arrow, `=` or keyword)");
                return;
            }
            for part in split_commas(head).into_iter().filter(|p| !p.is_empty()) {
                let i = self.resolve(&joined(part, src));
                self.node_attrs(i, &attrs, src);
                self.assign(i);
            }
            return;
        };
        let colon = head[fa..]
            .iter()
            .position(|t| t.kind == Kind::Colon)
            .map(|p| p + fa);
        let (chain_toks, label) = match colon {
            Some(c) => (&head[..c], joined(&head[c + 1..], src)),
            None => (head, String::new()),
        };
        let mut parts: Vec<&[Tok]> = vec![];
        let mut arrows = vec![];
        let mut from = 0;
        for (i, t) in chain_toks.iter().enumerate() {
            if let Kind::Arrow(a) = t.kind {
                parts.push(&chain_toks[from..i]);
                arrows.push(a);
                from = i + 1;
            }
        }
        parts.push(&chain_toks[from..]);
        let mut ends: Vec<Vec<usize>> = vec![];
        for part in &parts {
            let mut v = vec![];
            for ep in split_commas(part).into_iter().filter(|p| !p.is_empty()) {
                let i = self.resolve(&joined(ep, src));
                self.assign(i);
                v.push(i);
            }
            if v.is_empty() {
                self.warn("an arrow is missing a node on one side");
            }
            ends.push(v);
        }
        let mut line = LineStyle::Solid;
        let mut arrow_override = None;
        let mut role = None;
        for a in &attrs {
            let v = joined(a.toks, src).to_lowercase();
            match a.key.as_str() {
                "line" if !a.flag => match v.as_str() {
                    "solid" => line = LineStyle::Solid,
                    "dashed" => line = LineStyle::Dashed,
                    "dotted" => line = LineStyle::Dotted,
                    _ => self.warn(format!(
                        "unknown line style `{v}`; use solid, dashed or dotted"
                    )),
                },
                "arrow" if !a.flag => match v.as_str() {
                    "end" => arrow_override = Some(Arrow::End),
                    "both" => arrow_override = Some(Arrow::Both),
                    "none" => arrow_override = Some(Arrow::None),
                    _ => self.warn(format!("unknown arrow `{v}`; use end, both or none")),
                },
                "role" if !a.flag => match Role::from_name(&v) {
                    Some(r) => role = Some(r),
                    None => self.warn(format!("unknown role `{v}`")),
                },
                k => self.warn(format!("unknown or incomplete edge attribute `{k}`")),
            }
        }
        let mut made = 0usize;
        for (i, arrow) in arrows.iter().enumerate() {
            if ends[i].len() * ends[i + 1].len() + made > MAX_FANOUT {
                self.diag(Severity::Error, format!("this statement expands to more than {MAX_FANOUT} edges; the rest was skipped"));
                return;
            }
            for &f in &ends[i] {
                for &t in &ends[i + 1] {
                    self.edge_n += 1;
                    made += 1;
                    let mut e = Edge::new(
                        format!("e{}", self.edge_n),
                        self.d.nodes[f].id.clone(),
                        self.d.nodes[t].id.clone(),
                    );
                    e.label = label.clone();
                    e.line = line;
                    e.arrow = arrow_override.unwrap_or(*arrow);
                    e.role = role;
                    self.d.edges.push(e);
                }
            }
        }
    }
}

fn looks_iso(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        })
        && matches!(s[5..7].parse::<u32>(), Ok(1..=12))
        && matches!(s[8..10].parse::<u32>(), Ok(1..=31))
}
