//! Mermaid `graph` / `flowchart` import.
use std::collections::{HashMap, HashSet};

use rovelo_core::brand::Role;
use rovelo_core::model::{Arrow, Diagram, Direction, Edge, Group, LineStyle, Node, Shape};

use crate::parse::{MAX_LINES, slug};
use crate::{Diagnostic, Parsed, Severity};

pub fn from_mermaid(src: &str) -> Parsed {
    let mut m = Importer::default();
    m.run(src);
    m.finish()
}

#[derive(Default)]
struct Importer {
    d: Diagram,
    diags: Vec<Diagnostic>,
    line: usize,
    node_idx: HashMap<String, usize>,
    group_idx: HashMap<String, usize>,
    /// Ids written after `subgraph`, so an edge to one can be recognised.
    sub_ids: HashSet<String>,
    /// Open subgraphs, innermost last.
    stack: Vec<usize>,
}

struct Link {
    line: LineStyle,
    arrow: Arrow,
    label: String,
    invisible: bool,
    end: usize,
}

const DROPPED: [&str; 8] = [
    "classDef",
    "class",
    "style",
    "linkStyle",
    "click",
    "accTitle",
    "accDescr",
    "callback",
];

fn skip_ws(c: &[char], mut i: usize) -> usize {
    while c.get(i).is_some_and(|ch| ch.is_whitespace()) {
        i += 1;
    }
    i
}

fn at(c: &[char], i: usize, pat: &str) -> bool {
    pat.chars()
        .enumerate()
        .all(|(k, p)| c.get(i + k) == Some(&p))
}

fn find(c: &[char], from: usize, pat: &str) -> Option<usize> {
    (from..c.len()).find(|&i| at(c, i, pat))
}

fn run_of(c: &[char], mut i: usize, ch: char) -> usize {
    while c.get(i) == Some(&ch) {
        i += 1;
    }
    i
}

/// Mermaid label text: `<br/>` becomes a newline, quotes, backticks and a few entities are removed.
fn clean_text(s: &str) -> String {
    let mut out = String::new();
    let lower = s.to_ascii_lowercase();
    let mut i = 0;
    while i < s.len() {
        if lower[i..].starts_with("<br")
            && let Some(end) = lower[i..].find('>')
        {
            out.push('\n');
            i += end + 1;
            continue;
        }
        let Some(ch) = s[i..].chars().next() else {
            break;
        };
        out.push(ch);
        i += ch.len_utf8();
    }
    let out = out
        .replace("#quot;", "\"")
        .replace("&quot;", "\"")
        .replace("#amp;", "&")
        .replace("&amp;", "&")
        .replace("#lt;", "<")
        .replace("&lt;", "<")
        .replace("#gt;", ">")
        .replace("&gt;", ">");
    let t = out.trim();
    let t = t
        .strip_prefix('"')
        .and_then(|x| x.strip_suffix('"'))
        .unwrap_or(t);
    let t = t
        .strip_prefix('`')
        .and_then(|x| x.strip_suffix('`'))
        .unwrap_or(t);
    t.trim().to_string()
}

/// Split on `;` outside quotes and brackets.
fn split_statements(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let (mut q, mut depth) = (false, 0i32);
    for ch in line.chars() {
        match ch {
            '"' => q = !q,
            '[' | '(' | '{' if !q => depth += 1,
            ']' | ')' | '}' if !q => depth = (depth - 1).max(0),
            ';' if !q && depth == 0 => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    out.push(cur);
    out
}

/// `(id, title)` from the text after `subgraph`.
fn subgraph_header(rest: &str) -> (String, String) {
    let rest = rest.trim();
    if rest.is_empty() {
        return ("group".into(), "Group".into());
    }
    if let (Some(open), Some(close)) = (rest.find('['), rest.rfind(']'))
        && open < close
    {
        let id = rest[..open].trim();
        let title = clean_text(&rest[open + 1..close]);
        let id = if id.is_empty() {
            slug(&title)
        } else {
            id.to_string()
        };
        return (id, title);
    }
    let title = clean_text(rest);
    let single = !rest.contains(char::is_whitespace) && !rest.contains('"');
    (
        if single {
            rest.to_string()
        } else {
            slug(&title)
        },
        title,
    )
}

fn is_id_char(c: &[char], i: usize) -> bool {
    match c.get(i) {
        Some(ch) if ch.is_alphanumeric() || *ch == '_' => true,
        // A single dash inside an id (`my-node`), but never the start of a link.
        Some('-') => {
            i > 0
                && c.get(i - 1)
                    .is_some_and(|p| p.is_alphanumeric() || *p == '_')
                && c.get(i + 1)
                    .is_some_and(|n| n.is_alphanumeric() || *n == '_')
        }
        _ => false,
    }
}

impl Importer {
    fn diag(&mut self, severity: Severity, message: impl Into<String>) {
        self.diags.push(Diagnostic {
            line: self.line.max(1),
            severity,
            message: message.into(),
        });
    }

    fn warn(&mut self, m: impl Into<String>) {
        self.diag(Severity::Warning, m);
    }

    fn info(&mut self, m: impl Into<String>) {
        self.diag(Severity::Info, m);
    }

    fn finish(mut self) -> Parsed {
        // Pills are start/end markers. Only a first node nothing points at is "start" (success).
        let n = self.d.nodes.len();
        let mut incoming = vec![false; n];
        for e in &self.d.edges {
            if let Some(&t) = self.node_idx.get(&e.to) {
                incoming[t] = true;
            }
        }
        for (i, node) in self.d.nodes.iter_mut().enumerate() {
            match node.shape {
                Shape::Diamond => node.role = Role::Warning,
                Shape::Pill if i == 0 && !incoming[0] => node.role = Role::Success,
                _ => {}
            }
        }
        Parsed {
            diagram: self.d,
            diagnostics: self.diags,
        }
    }

    fn run(&mut self, src: &str) {
        let src = src.strip_prefix('\u{feff}').unwrap_or(src);
        let mut lines: Vec<(usize, &str)> = Vec::new();
        for (i, l) in src.lines().enumerate() {
            if i >= MAX_LINES {
                self.line = MAX_LINES + 1;
                self.diag(
                    Severity::Error,
                    format!("input is longer than {MAX_LINES} lines; the rest was ignored"),
                );
                break;
            }
            lines.push((i + 1, l));
        }
        let mut pos = 0;
        // Optional front matter: `---` ... `---`, of which only `title:` is used.
        let first = lines.iter().position(|(_, l)| !l.trim().is_empty());
        if let Some(f) = first.filter(|&f| lines[f].1.trim() == "---") {
            pos = f + 1;
            while pos < lines.len() && lines[pos].1.trim() != "---" {
                if let Some(t) = lines[pos].1.trim().strip_prefix("title:") {
                    self.d.title = clean_text(t);
                }
                pos += 1;
            }
            pos += 1;
        }
        let mut stmts: Vec<(usize, String)> = Vec::new();
        let mut header: Option<(usize, String)> = None;
        for &(n, l) in lines.iter().skip(pos) {
            let t = l.trim();
            if t.is_empty() || t.starts_with("%%") {
                continue;
            }
            for s in split_statements(t) {
                let s = s.trim();
                if s.is_empty() {
                    continue;
                }
                if header.is_none() {
                    header = Some((n, s.to_string()));
                } else {
                    stmts.push((n, s.to_string()));
                }
            }
        }
        let Some((hl, h)) = header else {
            self.line = 1;
            self.diag(Severity::Error, "no Mermaid diagram found");
            return;
        };
        self.line = hl;
        let mut words = h.split_whitespace();
        let kind = words.next().unwrap_or("").to_ascii_lowercase();
        if kind != "graph" && kind != "flowchart" && kind != "flowchart-elk" {
            let shown = h.split_whitespace().next().unwrap_or("");
            self.diag(Severity::Error, format!("`{shown}` is not a flowchart; only Mermaid `graph` and `flowchart` can be imported"));
            return;
        }
        match words.next() {
            None => {}
            Some(dir) => match Direction::from_name(&dir.to_ascii_uppercase()) {
                Some(d) => self.d.direction = d,
                None => self.warn(format!("unknown direction `{dir}`; using top to bottom")),
            },
        }
        for (_, s) in &stmts {
            if let Some(rest) = s
                .strip_prefix("subgraph")
                .filter(|r| r.is_empty() || r.starts_with(char::is_whitespace))
            {
                self.sub_ids.insert(subgraph_header(rest).0);
            }
        }
        for (n, s) in stmts {
            self.line = n;
            self.statement(&s);
        }
        for gi in std::mem::take(&mut self.stack) {
            let id = self.d.groups[gi].id.clone();
            self.warn(format!("subgraph `{id}` is never closed with `end`"));
        }
    }

    fn statement(&mut self, s: &str) {
        let first = s.split_whitespace().next().unwrap_or("");
        if first == "subgraph" {
            return self.open_subgraph(&s["subgraph".len()..]);
        }
        if s == "end" {
            if self.stack.pop().is_none() {
                self.warn("`end` closes nothing");
            }
            return;
        }
        if first == "direction" {
            self.warn("`direction` is ignored; the whole diagram uses its header direction");
            return;
        }
        if DROPPED.contains(&first) {
            self.info(format!(
                "`{first}` styling is not imported; roles and the brand kit style the result"
            ));
            return;
        }
        self.flow(s);
    }

    fn open_subgraph(&mut self, rest: &str) {
        let (id, title) = subgraph_header(rest);
        if !self.stack.is_empty() {
            self.warn(format!(
                "subgraph `{id}` is nested; groups do not nest, so it becomes a separate group"
            ));
        }
        // Group ids are slugs (`Backend` -> `backend`) so they serialize in the short `group Title {` form.
        let base = match slug(&id) {
            s if s.is_empty() => "group".to_string(),
            s => s,
        };
        let mut unique = base.clone();
        let mut n = 2;
        while self.group_idx.contains_key(&unique) {
            unique = format!("{base}-{n}");
            n += 1;
        }
        let gi = self.d.groups.len();
        self.group_idx.insert(unique.clone(), gi);
        self.d.groups.push(Group {
            id: unique,
            label: title,
            role: Role::Neutral,
        });
        self.stack.push(gi);
    }

    fn assign(&mut self, ni: usize) {
        let Some(&gi) = self.stack.last() else { return };
        let gid = self.d.groups[gi].id.clone();
        match &self.d.nodes[ni].group {
            None => self.d.nodes[ni].group = Some(gid),
            Some(g) if *g != gid => {
                let nid = self.d.nodes[ni].id.clone();
                self.warn(format!(
                    "node `{nid}` is in more than one subgraph; keeping the first"
                ));
            }
            Some(_) => {}
        }
    }

    // ---- one flowchart statement -------------------------------------------------------------

    fn flow(&mut self, s: &str) {
        let c: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut prev: Option<Vec<Option<usize>>> = None;
        let mut pending: Option<Link> = None;
        loop {
            let Some((group, ni)) = self.node_group(&c, i) else {
                if prev.is_some() {
                    self.warn(format!("a link in `{s}` has no node after it"));
                } else {
                    self.warn(format!("could not read `{s}`"));
                }
                return;
            };
            i = ni;
            if let (Some(p), Some(l)) = (&prev, &pending)
                && !l.invisible
            {
                for &f in p.iter().flatten() {
                    for &t in group.iter().flatten() {
                        let mut e = Edge::new(
                            format!("e{}", self.d.edges.len() + 1),
                            self.d.nodes[f].id.clone(),
                            self.d.nodes[t].id.clone(),
                        );
                        e.label = l.label.clone();
                        e.line = l.line;
                        e.arrow = l.arrow;
                        self.d.edges.push(e);
                    }
                }
            }
            i = skip_ws(&c, i);
            if i >= c.len() {
                return;
            }
            let Some(link) = self.link(&c, i) else {
                let rest: String = c[i..].iter().collect();
                self.warn(format!(
                    "could not read the link at `{}`",
                    rest.chars().take(20).collect::<String>()
                ));
                return;
            };
            i = link.end;
            pending = Some(link);
            prev = Some(group);
        }
    }

    /// `A & B & C`; an entry is `None` when it names a subgraph (edges to subgraphs are not supported).
    fn node_group(&mut self, c: &[char], mut i: usize) -> Option<(Vec<Option<usize>>, usize)> {
        let mut v = Vec::new();
        loop {
            i = skip_ws(c, i);
            let (r, ni) = self.node(c, i)?;
            v.push(r);
            i = skip_ws(c, ni);
            if c.get(i) == Some(&'&') {
                i += 1;
            } else {
                return Some((v, i));
            }
        }
    }

    fn node(&mut self, c: &[char], start: usize) -> Option<(Option<usize>, usize)> {
        let mut i = start;
        while is_id_char(c, i) {
            i += 1;
        }
        if i == start {
            return None;
        }
        let id: String = c[start..i].iter().collect();
        // `A@{ shape: x, label: "y" }` (newer syntax): read the label, drop the rest.
        let mut text: Option<String> = None;
        let mut shape: Option<Shape> = None;
        let j = skip_ws(c, i);
        if at(c, j, "@{") {
            let close = find(c, j, "}").unwrap_or(c.len());
            let body: String = c[j + 2..close.min(c.len())].iter().collect();
            if let Some(p) = body.find("label:") {
                let v = body[p + 6..].split(',').next().unwrap_or("");
                text = Some(clean_text(v));
            }
            self.warn(format!(
                "`{id}@{{...}}` shape syntax is not supported; the node is a plain rectangle"
            ));
            i = (close + 1).min(c.len());
        } else {
            let j = skip_ws(c, i);
            let opener = matches!(c.get(j), Some('[' | '(' | '{')) || at(c, i, ">");
            if opener {
                let o = if at(c, i, ">") { i } else { j };
                if let Some((sh, closers, open_len, note)) = shape_at(c, o) {
                    let (t, end, found) = read_text(c, o + open_len, closers);
                    if !found {
                        self.warn(format!("the text of node `{id}` is never closed"));
                    }
                    if let Some(n) = note {
                        self.info(format!("node `{id}`: {n}"));
                    }
                    text = Some(clean_text(&t));
                    shape = Some(sh);
                    i = end;
                }
            }
        }
        if at(c, i, ":::") {
            i += 3;
            while is_id_char(c, i) {
                i += 1;
            }
            self.info(format!("class shorthand on node `{id}` is not imported"));
        }
        if !self.node_idx.contains_key(&id) && self.sub_ids.contains(&id) && text.is_none() {
            self.warn(format!(
                "`{id}` is a subgraph; edges to subgraphs are not supported and are skipped"
            ));
            return Some((None, i));
        }
        Some((Some(self.upsert(&id, text, shape)), i))
    }

    fn upsert(&mut self, id: &str, text: Option<String>, shape: Option<Shape>) -> usize {
        let idx = match self.node_idx.get(id).copied() {
            Some(i) => i,
            None => {
                let i = self.d.nodes.len();
                self.node_idx.insert(id.to_string(), i);
                self.d.nodes.push(Node::new(id, id));
                i
            }
        };
        if let Some(t) = text {
            self.d.nodes[idx].label = if t.is_empty() { id.to_string() } else { t };
        }
        if let Some(s) = shape {
            self.d.nodes[idx].shape = s;
        }
        self.assign(idx);
        idx
    }

    fn link(&mut self, c: &[char], start: usize) -> Option<Link> {
        let mut j = start;
        // Edge ids (`e1@-->`) carry no information we keep.
        let mut k = j;
        while c
            .get(k)
            .is_some_and(|ch| ch.is_alphanumeric() || *ch == '_')
        {
            k += 1;
        }
        if k > j && c.get(k) == Some(&'@') && matches!(c.get(k + 1), Some('-' | '=' | '~')) {
            j = k + 1;
        }
        let mut left = false;
        let mut left_marker = false;
        match c.get(j) {
            Some('<') => {
                left = true;
                j += 1;
            }
            Some('o' | 'x') if matches!(c.get(j + 1), Some('-' | '=')) => {
                left = true;
                left_marker = true;
                j += 1;
            }
            _ => {}
        }
        let mut text: Option<String> = None;
        let (line, mut arrow, mut end);
        let mut note = false;
        // The character after a link body: `>` arrow head, `o`/`x` circle or cross head, or nothing.
        let head = |c: &[char], p: usize, note: &mut bool| -> (Arrow, usize) {
            match c.get(p) {
                Some('>') => (Arrow::End, p + 1),
                Some('o' | 'x') if c.get(p + 1).is_none_or(|n| n.is_whitespace() || *n == '|') => {
                    *note = true;
                    (Arrow::End, p + 1)
                }
                _ => (Arrow::None, p),
            }
        };
        match c.get(j)? {
            '~' => {
                let e = run_of(c, j, '~');
                if e - j < 3 {
                    return None;
                }
                self.info("invisible links (~~~) only affect Mermaid's layout and are dropped");
                return Some(Link {
                    line: LineStyle::Solid,
                    arrow: Arrow::None,
                    label: String::new(),
                    invisible: true,
                    end: e,
                });
            }
            '-' if c.get(j + 1) == Some(&'.') => {
                line = LineStyle::Dashed;
                let p = run_of(c, j + 1, '.');
                if c.get(p) == Some(&'-') {
                    (arrow, end) = head(c, p + 1, &mut note);
                } else if c.get(p).is_some_and(|ch| ch.is_whitespace()) {
                    let k = find(c, p, ".-")?;
                    let t: String = c[p..k].iter().collect();
                    text = Some(t.trim().trim_end_matches('.').trim().to_string());
                    (arrow, end) = head(c, k + 2, &mut note);
                } else {
                    return None;
                }
            }
            ch @ ('-' | '=') => {
                let ch = *ch;
                line = LineStyle::Solid;
                let p = run_of(c, j, ch);
                let n = p - j;
                if n < 2 {
                    return None;
                }
                if c.get(p) == Some(&'>')
                    || n >= 3
                    || matches!(c.get(p), Some('o' | 'x'))
                        && c.get(p + 1).is_none_or(|n| n.is_whitespace() || *n == '|')
                {
                    (arrow, end) = head(c, p, &mut note);
                } else if c.get(p).is_some_and(|x| x.is_whitespace()) {
                    // `-- text -->`
                    let pat = if ch == '-' { "--" } else { "==" };
                    let k = find(c, p, pat)?;
                    text = Some(c[p..k].iter().collect::<String>().trim().to_string());
                    let q = run_of(c, k, ch);
                    (arrow, end) = head(c, q, &mut note);
                } else {
                    return None;
                }
            }
            _ => return None,
        }
        if left {
            if left_marker && arrow == Arrow::End {
                note = true;
            }
            arrow = Arrow::Both;
        }
        if note {
            self.warn("circle and cross link ends are drawn as plain arrows");
        }
        // `|label|` after the link.
        let q = skip_ws(c, end);
        let mut label = text.map(|t| clean_text(&t)).unwrap_or_default();
        if c.get(q) == Some(&'|') {
            let close = (q + 1..c.len()).find(|&x| c[x] == '|');
            match close {
                Some(close) => {
                    label = clean_text(&c[q + 1..close].iter().collect::<String>());
                    end = close + 1;
                }
                None => {
                    self.warn("link label is missing its closing `|`");
                    label = clean_text(&c[q + 1..].iter().collect::<String>());
                    end = c.len();
                }
            }
        }
        Some(Link {
            line,
            arrow,
            label,
            invisible: false,
            end,
        })
    }
}

type ShapeAt = (Shape, &'static [&'static str], usize, Option<&'static str>);

fn shape_at(c: &[char], i: usize) -> Option<ShapeAt> {
    let table: [(&str, Shape, &'static [&'static str], Option<&'static str>); 12] = [
        (
            "(((",
            Shape::Circle,
            &[")))"],
            Some("double circle is drawn as a circle"),
        ),
        ("((", Shape::Circle, &["))"], None),
        ("([", Shape::Pill, &["])"], None),
        ("[(", Shape::Cylinder, &[")]"], None),
        (
            "[[",
            Shape::Rect,
            &["]]"],
            Some("subroutine shape is drawn as a rectangle"),
        ),
        ("{{", Shape::Hexagon, &["}}"], None),
        ("[/", Shape::Parallelogram, &["/]", "\\]"], None),
        ("[\\", Shape::Parallelogram, &["/]", "\\]"], None),
        (">", Shape::Note, &["]"], None),
        ("(", Shape::Rounded, &[")"], None),
        ("{", Shape::Diamond, &["}"], None),
        ("[", Shape::Rect, &["]"], None),
    ];
    table
        .into_iter()
        .find(|(p, ..)| at(c, i, p))
        .map(|(p, s, closers, note)| (s, closers, p.chars().count(), note))
}

/// Text up to the first closer; a quoted text may hold the closer itself.
fn read_text(c: &[char], start: usize, closers: &[&str]) -> (String, usize, bool) {
    let first_closer = |from: usize| {
        (from..c.len()).find_map(|i| {
            closers
                .iter()
                .find(|p| at(c, i, p))
                .map(|p| (i, p.chars().count()))
        })
    };
    let q = skip_ws(c, start);
    if c.get(q) == Some(&'"')
        && let Some(close) = (q + 1..c.len()).find(|&x| c[x] == '"')
    {
        let after = skip_ws(c, close + 1);
        if let Some(p) = closers.iter().find(|p| at(c, after, p)) {
            return (
                c[q..=close].iter().collect(),
                after + p.chars().count(),
                true,
            );
        }
    }
    match first_closer(start) {
        Some((i, n)) => (c[start..i].iter().collect(), i + n, true),
        None => (c[start..].iter().collect(), c.len(), false),
    }
}
