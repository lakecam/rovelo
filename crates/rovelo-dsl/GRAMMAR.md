# rovelo language, version 1

One statement per line, forgiving: any text gives the best diagram it can plus diagnostics, never a panic.
`parse(text)` and `serialize(diagram)` are inverses up to node positions and edge ids (see Laws).

## Lexical rules

- Lines: `\n` or `\r\n`; a leading BOM is ignored. Hard limit 20,000 lines: one error diagnostic, then parsing stops.
- `;` also separates statements on a line. `{` and `}` open and close a `group` block.
- Comments: `//` to end of line, but only at the line start or after whitespace and outside quotes, so `https://x.y` survives.
  `%% rovelo 1` is the version pragma; any other `%%` line is ignored.
- Reserved: `->` `<->` `--` (also `-->`, `<-->`, `---`) `:` `=` `,` `;` `[` `]` `{` `}` `"` and `//`.
  A single `-` is an ordinary character (`Wi-Fi`, `2026-11-10`).
- A bare label is the trimmed source text up to the next reserved piece; inner spacing is kept. Quote with `"..."` when a
  label has reserved punctuation, leading/trailing/double spaces, control characters, or starts with a keyword
  (`group todo title brand flowchart userflow taskflow`, any case). Escapes: `\"` `\\` `\n` `\r` `\t` `\u{hex}`.
  The serializer quotes exactly those labels.
- Ids (node, group, task) are single tokens: quoted if they contain whitespace or any reserved piece.

## Statements

```
%% rovelo 1                     version pragma (serialize always writes it first)
taskflow right                  family line: flowchart | userflow | taskflow, then down|right|left|up (TB TD LR RL BT too)
title: Onboarding               rest of the line is the title (quote it to keep edge whitespace)
brand: midnight                 preset (rovelo | slate | paper | midnight); resets the kit, so write it before overrides
brand.<key>: <value>            overrides, see below
id = Label [attrs]              declare a node; declaring an existing id updates it
Label [attrs]                   bare node; attrs apply to the node
a -> b                          edge; ids first, then labels, else a new node (id = slug of the label, made unique)
a -> b -> c                     chain: one edge per hop
a -> b, c      a, b -> c        fan-out / fan-in: one edge per pair
a -> b: Edge label [attrs]      label after ':' ; attrs apply to every edge the statement makes
group Auth { a; b }             block group; members are the nodes the block mentions
group id = "Label" [role: x] {  explicit id; an existing explicit id is reopened, not duplicated
  c
}
todo node: Title [attrs]        task linked to a node (id or label)
todo: Title [attrs]             free-floating task
```

Arrows: `->` end arrow, `<->` both ends, `--` none. Groups do not nest (a nested group is a warning and becomes a separate group);
a node has one group (a second mention moves it, with a warning). Edges to unknown names create nodes; a `todo` for an unknown node
creates the node and warns.

### Attributes

`[name: value, flag, ...]`, values bare or quoted. A bare name is a flag (`done`).

| On | Attribute | Values |
|---|---|---|
| node | `shape` | rect rounded pill diamond circle parallelogram hexagon cylinder document screen note |
| node | `role` | primary secondary accent neutral success warning danger (default primary) |
| node | `icon` | an icon word, e.g. `user` |
| node | `size` | `WxH` manual size override |
| edge | `line` | solid dashed dotted |
| edge | `arrow` | end both none (overrides the arrow token) |
| edge | `role` | any role (default: none, drawn neutral) |
| group | `role` | any role (default neutral) |
| todo | `id` | task id; default `t1`, `t2`, ... in order |
| todo | `done` | flag, or `done: false` |
| todo | `due` | `YYYY-MM-DD` (other text is kept, with a warning) |
| todo | `priority` | low normal high |
| todo | `tags` | space-separated words; quote a tag that has spaces: `tags: copy "user research"` |
| todo | `notes` | free text, quoted |

Unknown attributes or values produce a warning on their line and are skipped.

### Brand keys

`brand.name`, the seven roles (`brand.primary` `secondary` `accent` `neutral` `success` `warning` `danger`), `brand.background`,
`brand.surface`, `brand.text`, `brand.muted_text` (colours as `#rgb` or `#rrggbb`), `brand.font`, `brand.heading_font`,
`brand.radius`, `brand.stroke`, `brand.size` (numbers), `brand.routing` (straight orthogonal curved), `brand.fill`
(solid soft outline), `brand.nodes` (card shape: an icon-tile card, or the classic outline shapes), `brand.shadow` and
`brand.grid` (on off), `brand.logo` (quoted inline SVG, or `none`).
Values of these lines run to the end of the line, so `brand.font: Inter, sans-serif` needs no quotes.

## Laws

1. `serialize(parse(x))` is a fixpoint: parsing and serializing it again changes nothing.
2. For any diagram `d` (ids any strings, labels any Unicode), `parse(serialize(d)).diagram` equals `d`
   except node `pos` (always `None`) and edge ids (renumbered `e1`, `e2`, ...). Node, edge, group and task order, and the
   whole brand kit, survive.

Serializer layout: header (`%% rovelo 1`, family line, `title`, `brand` and only the brand keys that differ from the named
preset; a kit whose name is not a preset gets `brand.name` plus every key), then nodes, edges, groups, todos. Every node is
declared as `id = Label`, so edges and groups only ever mention ids.

## Example 1: a user flow

```
%% rovelo 1
userflow right
title: Sign-up
brand: paper

landing = Landing page [shape: screen, icon: home]
form = Sign-up form [shape: screen]
ok = Welcome [shape: screen, role: success]

landing -> form: Tap Get started
form -> ok: Valid details
form -> form: Invalid details [line: dashed, role: danger]

group Onboarding {
  form
  ok
}
```

## Example 2: a task flow with todos

Hand-written, using shortcuts the serializer expands:

```
taskflow down
title: Launch checklist
Write copy -> Review -> Ship
group Content { Write copy; Review }
todo write-copy: Draft headline options [priority: high, tags: copy ux, due: 2026-11-10]
todo review: Get sign-off from legal [notes: "see brief"]
todo ship: Tag the release [done]
todo: Book the launch party
```

becomes, from `serialize`:

```
%% rovelo 1
taskflow down
title: Launch checklist
brand: rovelo

write-copy = Write copy
review = Review
ship = Ship

write-copy -> review
review -> ship

group Content {
  write-copy
  review
}

todo write-copy: Draft headline options [due: 2026-11-10, priority: high, tags: copy ux]
todo review: Get sign-off from legal [notes: "see brief"]
todo ship: Tag the release [done]
todo: Book the launch party
```

## Example 3: Mermaid conversion

`from_mermaid` reads `graph` / `flowchart` (TB TD LR RL BT). Shapes: `[]` rect, `()` rounded, `([])` pill, `{}` diamond,
`(())` circle, `[()]` cylinder, `{{}}` hexagon, `[/ /]` `[\ \]` parallelogram, `>]` note, `[[]]` rect (info). Links: `-->` `---`
`-.->` `-.-` `==>` `===` `<-->`, `--o`/`--x` (warning, drawn as arrows), labels `-->|t|`, `-- t -->`, `-. t .->`, chains and `&`.
`<br/>` in text becomes a newline. `subgraph` becomes a group (nested ones are flattened with a warning). `classDef`, `class`,
`style`, `linkStyle` and `click` lines are dropped, one info diagnostic each; `direction` inside a subgraph warns. Roles:
diamonds get `warning`; a pill is `success` only if it is the first node and nothing points at it, otherwise `primary`.
Anything that is not a flowchart is one error and an empty diagram.

```
flowchart TD
  A([Start]) --> B[Enter email]
  B --> C{Valid?}
  C -->|yes| D[(Save user)]
  C -->|no| B
  subgraph Backend
    D --> E((Done))
  end
  classDef hot fill:#f96
```

gives (plus one info diagnostic for `classDef`):

```
%% rovelo 1
flowchart down
title: Untitled
brand: rovelo

A = Start [shape: pill, role: success]
B = Enter email
C = Valid? [shape: diamond, role: warning]
D = Save user [shape: cylinder]
E = Done [shape: circle]

A -> B
B -> C
C -> D: yes
C -> B: no
D -> E

group Backend {
  D
  E
}
```
