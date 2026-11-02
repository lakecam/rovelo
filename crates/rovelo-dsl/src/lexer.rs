//! Line lexer and quoting rules shared by the parser and the serializer.
use rovelo_core::model::Arrow;

/// Words that open a statement; labels that start with one are quoted on output.
pub const KEYWORDS: [&str; 7] = [
    "group",
    "todo",
    "title",
    "brand",
    "flowchart",
    "userflow",
    "taskflow",
];

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Word,
    Str(String),
    Arrow(Arrow),
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Semi,
    Comma,
    Colon,
    Eq,
}

/// A token with its byte span in the line it came from.
#[derive(Clone, Debug)]
pub struct Tok {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

pub struct Lexed {
    pub toks: Vec<Tok>,
    pub unterminated: bool,
}

/// Cut a `//` comment. It needs whitespace (or the line start) before it and must be outside quotes,
/// so `https://x` survives.
pub fn strip_comment(line: &str) -> &str {
    let mut in_q = false;
    let mut esc = false;
    let mut prev_ws = true;
    let mut it = line.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if in_q {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_q = false;
            }
        } else if c == '"' {
            in_q = true;
        } else if c == '/' && prev_ws && matches!(it.peek(), Some((_, '/'))) {
            return &line[..i];
        }
        prev_ws = c.is_whitespace();
    }
    line
}

/// `->`/`-->` (end), `<->`/`<-->` (both), `--`/`---` (none) starting at byte `i`.
fn arrow_at(s: &str, i: usize) -> Option<(Arrow, usize)> {
    let b = s.as_bytes();
    let mut j = i;
    let left = b.get(j) == Some(&b'<');
    if left {
        j += 1;
    }
    let ds = j;
    while b.get(j) == Some(&b'-') {
        j += 1;
    }
    let n = j - ds;
    if n == 0 {
        return None;
    }
    if b.get(j) == Some(&b'>') {
        return Some((if left { Arrow::Both } else { Arrow::End }, j + 1 - i));
    }
    if !left && n >= 2 {
        return Some((Arrow::None, j - i));
    }
    None
}

fn is_punct(c: char) -> bool {
    matches!(c, '"' | '[' | ']' | '{' | '}' | ';' | ',' | ':' | '=')
}

fn read_string(s: &str, start: usize) -> (String, usize, bool) {
    let mut out = String::new();
    let mut it = s[start + 1..].char_indices();
    while let Some((o, c)) = it.next() {
        let at = start + 1 + o;
        match c {
            '"' => return (out, at + 1, true),
            '\\' => match it.next() {
                Some((_, 'n')) => out.push('\n'),
                Some((_, 'r')) => out.push('\r'),
                Some((_, 't')) => out.push('\t'),
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, 'u')) => {
                    // `\u{hex}`; anything malformed stays literal.
                    let rest = &s[start + 1 + o + 2..];
                    let parsed = rest.strip_prefix('{').and_then(|r| {
                        let close = r.find('}')?;
                        let ch = u32::from_str_radix(&r[..close], 16)
                            .ok()
                            .and_then(char::from_u32)?;
                        Some((ch, close + 2))
                    });
                    match parsed {
                        Some((ch, used)) => {
                            out.push(ch);
                            for _ in 0..used {
                                it.next();
                            }
                        }
                        None => out.push_str("\\u"),
                    }
                }
                Some((_, other)) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            },
            c => out.push(c),
        }
    }
    (out, s.len(), false)
}

pub fn lex(s: &str) -> Lexed {
    let mut toks = Vec::new();
    let mut unterminated = false;
    let mut i = 0;
    while i < s.len() {
        let Some(c) = s[i..].chars().next() else {
            break;
        };
        if c.is_whitespace() {
            i += c.len_utf8();
            continue;
        }
        let start = i;
        let simple = match c {
            '[' => Some(Kind::LBracket),
            ']' => Some(Kind::RBracket),
            '{' => Some(Kind::LBrace),
            '}' => Some(Kind::RBrace),
            ';' => Some(Kind::Semi),
            ',' => Some(Kind::Comma),
            ':' => Some(Kind::Colon),
            '=' => Some(Kind::Eq),
            _ => None,
        };
        if let Some(kind) = simple {
            i += 1;
            toks.push(Tok {
                kind,
                start,
                end: i,
            });
            continue;
        }
        if c == '"' {
            let (val, end, ok) = read_string(s, i);
            unterminated |= !ok;
            toks.push(Tok {
                kind: Kind::Str(val),
                start,
                end,
            });
            i = end;
            continue;
        }
        if let Some((a, len)) = arrow_at(s, i) {
            i += len;
            toks.push(Tok {
                kind: Kind::Arrow(a),
                start,
                end: i,
            });
            continue;
        }
        // A word runs to whitespace or the next reserved piece.
        i += c.len_utf8();
        while i < s.len() {
            let Some(c) = s[i..].chars().next() else {
                break;
            };
            if c.is_whitespace() || is_punct(c) || arrow_at(s, i).is_some() {
                break;
            }
            i += c.len_utf8();
        }
        toks.push(Tok {
            kind: Kind::Word,
            start,
            end: i,
        });
    }
    Lexed { toks, unterminated }
}

/// Quote with `"`; only `\"` `\\` `\n` `\r` `\t` and `\u{..}` are escapes.
pub fn quote(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if c.is_control() => o.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// True when `s` would not lex back to itself as a bare name, label or attribute value.
pub fn needs_quote(s: &str) -> bool {
    if s.is_empty() || s.starts_with("%%") || s != s.trim() {
        return true;
    }
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_control() || is_punct(c) {
            return true;
        }
        if c.is_whitespace() {
            if c != ' ' || prev_space {
                return true;
            }
            prev_space = true;
        } else {
            prev_space = false;
        }
    }
    if s.contains("->") || s.contains("--") || s.contains("//") {
        return true;
    }
    let first = s.split(' ').next().unwrap_or("");
    KEYWORDS.iter().any(|k| first.eq_ignore_ascii_case(k))
}

pub fn name(s: &str) -> String {
    if needs_quote(s) {
        quote(s)
    } else {
        s.to_string()
    }
}

/// Like `name`, but for ids and tags, which must lex as a single token.
pub fn ident(s: &str) -> String {
    if s.chars().any(char::is_whitespace) || needs_quote(s) {
        quote(s)
    } else {
        s.to_string()
    }
}

/// Looser rule for whole-line values (`title: ...`, `brand.font: ...`) that are read raw to the end of line.
pub fn needs_quote_line(s: &str) -> bool {
    s.is_empty()
        || s != s.trim()
        || s.contains('"')
        || s.contains("//")
        || s.chars()
            .any(|c| c.is_control() || (c.is_whitespace() && c != ' '))
}

pub fn line_value(s: &str) -> String {
    if needs_quote_line(s) {
        quote(s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(s: &str) -> Vec<Kind> {
        lex(s).toks.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn comments_need_a_boundary_and_skip_quotes() {
        assert_eq!(strip_comment("a -> b // note"), "a -> b ");
        assert_eq!(strip_comment("// all"), "");
        assert_eq!(strip_comment("see https://x.y/z"), "see https://x.y/z");
        assert_eq!(strip_comment("\"a // b\" c"), "\"a // b\" c");
        assert_eq!(strip_comment("\"a \\\" // b\" // c"), "\"a \\\" // b\" ");
    }

    #[test]
    fn arrows_and_dashes() {
        assert_eq!(
            kinds("a->b <-> c -- d --> e"),
            vec![
                Kind::Word,
                Kind::Arrow(Arrow::End),
                Kind::Word,
                Kind::Arrow(Arrow::Both),
                Kind::Word,
                Kind::Arrow(Arrow::None),
                Kind::Word,
                Kind::Arrow(Arrow::End),
                Kind::Word
            ]
        );
        assert_eq!(kinds("Wi-Fi 2026-11-10 -5 -"), vec![Kind::Word; 4]);
    }

    #[test]
    fn strings_unescape() {
        let l = lex(r#""a\"b\\c\nd\u{e9}\q""#);
        assert_eq!(l.toks[0].kind, Kind::Str("a\"b\\c\nd\u{e9}\\q".into()));
        assert!(!l.unterminated);
        assert!(lex("\"open").unterminated);
    }

    #[test]
    fn quoting_round_trips_through_the_lexer() {
        for s in [
            "",
            " a",
            "a  b",
            "x\ny",
            "a\"b",
            "back\\slash",
            "tab\t",
            "\u{7}bell",
            "ünï ☃",
            "a, b",
        ] {
            let q = quote(s);
            let l = lex(&q);
            assert_eq!(l.toks.len(), 1, "{s:?}");
            assert_eq!(l.toks[0].kind, Kind::Str(s.to_string()), "{s:?}");
        }
    }

    #[test]
    fn quote_rules() {
        for s in [
            "a->b",
            "a--b",
            "a:b",
            "x [y]",
            "a;b",
            "a=b",
            "http://x",
            "group chat",
            "Todo",
            "%% x",
            "",
        ] {
            assert!(needs_quote(s), "{s:?}");
        }
        for s in ["Sign up", "Wi-Fi", "10", "ünï", "a < b", "-", "#3b5bdb"] {
            assert!(!needs_quote(s), "{s:?}");
        }
    }
}
