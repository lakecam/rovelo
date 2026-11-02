//! Automatic layout. Layered (Sugiyama-style): break cycles, rank, order, place.
use crate::geometry::{GROUP_PAD, GROUP_TITLE, node_size};
use crate::model::{Diagram, Direction, Point, Rect, Size};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

const RANK_GAP: f32 = 70.0;
const NODE_GAP: f32 = 50.0;
const DUMMY_GAP: f32 = 20.0;
const COMPONENT_GAP: f32 = 80.0;
/// Free space between a group's border and the nearest node outside it, along the flow.
const GROUP_GAP: f32 = 56.0;
/// Upper bound on virtual nodes standing in for long edges; further long edges are ignored
/// when ordering and placing (they still count for ranking).
const MAX_DUMMIES: usize = 4000;
const MIN_SWEEPS: usize = 4;
const MAX_SWEEPS: usize = 24;
const STALL_SWEEPS: usize = 4;
const PLACE_PASSES: usize = 8;

/// Position every node. Honours `diagram.direction`. Overwrites all `pos` values.
pub fn auto_layout(d: &mut Diagram) {
    let pos = compute(d);
    for (n, p) in d.nodes.iter_mut().zip(pos) {
        n.pos = Some(p);
    }
}

/// Position only nodes whose `pos` is `None`, leaving hand-placed nodes where they are.
/// New nodes land next to placed neighbours; the rest are set beside the existing diagram.
pub fn layout_unplaced(d: &mut Diagram) {
    if d.nodes.iter().all(|n| n.pos.is_some()) {
        return;
    }
    let virt = compute(d);
    let sizes: Vec<Size> = d.nodes.iter().map(|n| node_size(d, n)).collect();
    let mut pos: Vec<Option<Point>> = d.nodes.iter().map(|n| n.pos).collect();
    if pos.iter().all(Option::is_none) {
        for (n, p) in d.nodes.iter_mut().zip(virt) {
            n.pos = Some(p);
        }
        return;
    }
    let index: HashMap<&str, usize> = d
        .nodes
        .iter()
        .enumerate()
        .rev()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut adj: Vec<Vec<usize>> = vec![vec![]; d.nodes.len()];
    for e in &d.edges {
        if let (Some(&a), Some(&b)) = (index.get(e.from.as_str()), index.get(e.to.as_str()))
            && a != b
        {
            adj[a].push(b);
            adj[b].push(a);
        }
    }
    let rect_of = |p: Point, s: Size| Rect {
        x: p.x,
        y: p.y,
        w: s.w,
        h: s.h,
    };
    let mut occupied: Vec<Rect> = pos
        .iter()
        .zip(&sizes)
        .filter_map(|(p, s)| p.map(|p| rect_of(p, *s)))
        .collect();

    // Grow outward from the placed nodes: each new node keeps its layout offset to a placed neighbour.
    loop {
        let mut progress = false;
        for i in 0..pos.len() {
            if pos[i].is_some() {
                continue;
            }
            let anchors: Vec<Point> = adj[i]
                .iter()
                .filter_map(|&j| {
                    pos[j].map(|pj| {
                        Point::new(pj.x + virt[i].x - virt[j].x, pj.y + virt[i].y - virt[j].y)
                    })
                })
                .collect();
            if anchors.is_empty() {
                continue;
            }
            let k = anchors.len() as f32;
            let want = Point::new(
                anchors.iter().map(|p| p.x).sum::<f32>() / k,
                anchors.iter().map(|p| p.y).sum::<f32>() / k,
            );
            let at = free_spot(want, sizes[i], &occupied, d.direction);
            occupied.push(rect_of(at, sizes[i]));
            pos[i] = Some(at);
            progress = true;
        }
        if !progress {
            break;
        }
    }

    // Whatever has no placed neighbour forms its own blocks: park them beside the placed diagram.
    let orphans: Vec<usize> = (0..pos.len()).filter(|&i| pos[i].is_none()).collect();
    if !orphans.is_empty() {
        let placed = occupied
            .iter()
            .fold(None, |acc: Option<Rect>, r| {
                Some(acc.map_or(*r, |a| a.union(r)))
            })
            .unwrap_or_default();
        let min_x = orphans.iter().map(|&i| virt[i].x).fold(f32::MAX, f32::min);
        let min_y = orphans.iter().map(|&i| virt[i].y).fold(f32::MAX, f32::min);
        let (tx, ty) = if matches!(d.direction, Direction::Down | Direction::Up) {
            (
                placed.x + placed.w + COMPONENT_GAP - min_x,
                placed.y - min_y,
            )
        } else {
            (
                placed.x - min_x,
                placed.y + placed.h + COMPONENT_GAP - min_y,
            )
        };
        for &i in &orphans {
            let want = Point::new(virt[i].x + tx, virt[i].y + ty);
            let at = free_spot(want, sizes[i], &occupied, d.direction);
            occupied.push(rect_of(at, sizes[i]));
            pos[i] = Some(at);
        }
    }
    for (n, p) in d.nodes.iter_mut().zip(pos) {
        n.pos = p;
    }
}

/// `want` if a node of `size` fits there, else the nearest free spot found by sliding
/// across the flow, then along it.
fn free_spot(want: Point, size: Size, occupied: &[Rect], dir: Direction) -> Point {
    const PAD: f32 = 16.0;
    const STEP: f32 = 24.0;
    let fits = |p: Point| {
        let r = Rect {
            x: p.x - PAD,
            y: p.y - PAD,
            w: size.w + 2.0 * PAD,
            h: size.h + 2.0 * PAD,
        };
        occupied
            .iter()
            .all(|o| r.x >= o.x + o.w || o.x >= r.x + r.w || r.y >= o.y + o.h || o.y >= r.y + r.h)
    };
    if fits(want) {
        return want;
    }
    let (side, fwd) = match dir {
        Direction::Down => (Point::new(1.0, 0.0), Point::new(0.0, 1.0)),
        Direction::Up => (Point::new(1.0, 0.0), Point::new(0.0, -1.0)),
        Direction::Right => (Point::new(0.0, 1.0), Point::new(1.0, 0.0)),
        Direction::Left => (Point::new(0.0, 1.0), Point::new(-1.0, 0.0)),
    };
    for k in 1..=60 {
        let off = k as f32 * STEP;
        for sign in [1.0, -1.0] {
            let p = Point::new(want.x + side.x * off * sign, want.y + side.y * off * sign);
            if fits(p) {
                return p;
            }
        }
    }
    for k in 1..=60 {
        let off = k as f32 * STEP;
        let p = Point::new(want.x + fwd.x * off, want.y + fwd.y * off);
        if fits(p) {
            return p;
        }
    }
    want
}

/// Top-left position for every node, in node order. Pure function of the diagram.
fn compute(d: &Diagram) -> Vec<Point> {
    let n = d.nodes.len();
    if n == 0 {
        return vec![];
    }
    let sizes: Vec<Size> = d.nodes.iter().map(|nd| node_size(d, nd)).collect();
    let index: HashMap<&str, usize> = d
        .nodes
        .iter()
        .enumerate()
        .rev()
        .map(|(i, nd)| (nd.id.as_str(), i))
        .collect();
    let group_ix: HashMap<&str, usize> = d
        .groups
        .iter()
        .enumerate()
        .rev()
        .map(|(i, g)| (g.id.as_str(), i))
        .collect();
    let groups: Vec<Option<usize>> = d
        .nodes
        .iter()
        .map(|nd| nd.group.as_deref().and_then(|g| group_ix.get(g).copied()))
        .collect();
    let edges: Vec<(usize, usize)> = d
        .edges
        .iter()
        .filter_map(|e| Some((*index.get(e.from.as_str())?, *index.get(e.to.as_str())?)))
        .filter(|(a, b)| a != b)
        .collect();

    // Connected components, in order of their first node.
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    for &(a, b) in &edges {
        let (ra, rb) = (find(&mut parent, a), find(&mut parent, b));
        if ra != rb {
            parent[ra.max(rb)] = ra.min(rb);
        }
    }
    let mut comp_of_root: HashMap<usize, usize> = HashMap::new();
    let mut comps: Vec<Vec<usize>> = vec![];
    for i in 0..n {
        let r = find(&mut parent, i);
        let c = *comp_of_root.entry(r).or_insert_with(|| {
            comps.push(vec![]);
            comps.len() - 1
        });
        comps[c].push(i);
    }
    let mut comp_edges: Vec<Vec<(usize, usize)>> = vec![vec![]; comps.len()];
    for &(a, b) in &edges {
        let r = find(&mut parent, a);
        comp_edges[comp_of_root[&r]].push((a, b));
    }

    let mut local = vec![0usize; n];
    for c in &comps {
        for (li, &g) in c.iter().enumerate() {
            local[g] = li;
        }
    }
    let laid: Vec<(Vec<Point>, Size)> = comps
        .iter()
        .zip(&comp_edges)
        .map(|(nodes, es)| {
            let ls: Vec<Size> = nodes.iter().map(|&g| sizes[g]).collect();
            let lg: Vec<Option<usize>> = nodes.iter().map(|&g| groups[g]).collect();
            let le: Vec<(usize, usize)> = es.iter().map(|&(a, b)| (local[a], local[b])).collect();
            layer_component(&ls, &lg, &le, d.direction)
        })
        .collect();
    pack_components(&comps, &laid, n, d.direction)
}

/// Shelf-pack component layouts: side by side along the in-rank axis, wrapping when a row gets long.
fn pack_components(
    comps: &[Vec<usize>],
    laid: &[(Vec<Point>, Size)],
    n: usize,
    dir: Direction,
) -> Vec<Point> {
    let horizontal_rows = matches!(dir, Direction::Down | Direction::Up);
    let dims = |s: Size| {
        if horizontal_rows {
            (s.w, s.h)
        } else {
            (s.h, s.w)
        }
    };
    let area: f32 = laid
        .iter()
        .map(|(_, s)| (s.w + COMPONENT_GAP) * (s.h + COMPONENT_GAP))
        .sum();
    let widest = laid.iter().map(|(_, s)| dims(*s).0).fold(0.0, f32::max);
    let limit = (area.sqrt() * 1.6).max(1200.0).max(widest);
    let mut out = vec![Point::default(); n];
    let (mut along, mut across, mut thick) = (0.0f32, 0.0f32, 0.0f32);
    for (nodes, (pts, size)) in comps.iter().zip(laid) {
        let (len, th) = dims(*size);
        if along > 0.0 && along + len > limit {
            across += thick + COMPONENT_GAP;
            along = 0.0;
            thick = 0.0;
        }
        let (ox, oy) = if horizontal_rows {
            (along, across)
        } else {
            (across, along)
        };
        for (&g, p) in nodes.iter().zip(pts) {
            out[g] = Point::new(p.x + ox, p.y + oy);
        }
        along += len + COMPONENT_GAP;
        thick = thick.max(th);
    }
    out
}

/// One layered node: a real node or a virtual one on a long edge.
struct Lv {
    rank: usize,
    /// Extent across the rank (width for vertical flow).
    sec: f32,
    group: Option<usize>,
    dummy: bool,
}

/// Lay out one connected component; positions are relative to its own top-left corner.
fn layer_component(
    sizes: &[Size],
    groups: &[Option<usize>],
    edges: &[(usize, usize)],
    dir: Direction,
) -> (Vec<Point>, Size) {
    let m = sizes.len();
    let vertical = matches!(dir, Direction::Down | Direction::Up);
    let sec_of = |s: Size| if vertical { s.w } else { s.h };
    let prim_of = |s: Size| if vertical { s.h } else { s.w };

    let rank = rank_nodes(m, edges);
    let ranks = rank.iter().copied().max().unwrap_or(0) + 1;

    // Layered graph: real nodes first, then virtual nodes for edges spanning several ranks.
    let mut lv: Vec<Lv> = (0..m)
        .map(|i| Lv {
            rank: rank[i],
            sec: sec_of(sizes[i]),
            group: groups[i],
            dummy: false,
        })
        .collect();
    let dag = dag_edges(m, edges);
    let mut order: Vec<usize> = (0..dag.len()).collect();
    order.sort_by_key(|&i| (rank[dag[i].1] - rank[dag[i].0], i));
    let mut ledges: Vec<(usize, usize)> = vec![];
    let mut dummies = 0usize;
    for &i in &order {
        let (u, v) = dag[i];
        let span = rank[v] - rank[u];
        if span == 1 {
            ledges.push((u, v));
        } else if dummies + span - 1 <= MAX_DUMMIES {
            let mut prev = u;
            for r in rank[u] + 1..rank[v] {
                lv.push(Lv {
                    rank: r,
                    sec: 0.0,
                    group: None,
                    dummy: true,
                });
                let id = lv.len() - 1;
                ledges.push((prev, id));
                prev = id;
            }
            ledges.push((prev, v));
            dummies += span - 1;
        }
    }
    let total = lv.len();
    let mut preds: Vec<Vec<usize>> = vec![vec![]; total];
    let mut succs: Vec<Vec<usize>> = vec![vec![]; total];
    for &(u, v) in &ledges {
        succs[u].push(v);
        preds[v].push(u);
    }

    let mut layers = initial_order(&lv, &succs, &preds, ranks);
    order_layers(&mut layers, &lv, &preds, &succs, &ledges);
    let xs = assign_secondary(&layers, &lv, &preds, &succs);

    // Primary axis: ranks stacked with a fixed gap, each as thick as its biggest node.
    let mut thickness = vec![0.0f32; ranks];
    for i in 0..m {
        thickness[lv[i].rank] = thickness[lv[i].rank].max(prim_of(sizes[i]));
    }
    let gaps = rank_gaps(&rank, groups, ranks, dir);
    let mut centre_v = vec![0.0f32; ranks];
    let mut at = 0.0;
    for r in 0..ranks {
        centre_v[r] = at + thickness[r] / 2.0;
        at += thickness[r] + gaps.get(r).copied().unwrap_or(RANK_GAP);
    }

    let mut pts: Vec<Point> = (0..m)
        .map(|i| {
            let (cu, cv) = (xs[i], centre_v[lv[i].rank]);
            let (cx, cy) = match dir {
                Direction::Down => (cu, cv),
                Direction::Up => (cu, -cv),
                Direction::Right => (cv, cu),
                Direction::Left => (-cv, cu),
            };
            Point::new(cx - sizes[i].w / 2.0, cy - sizes[i].h / 2.0)
        })
        .collect();
    let min_x = pts.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let min_y = pts.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let mut max_x = 0.0f32;
    let mut max_y = 0.0f32;
    for (p, s) in pts.iter_mut().zip(sizes) {
        p.x -= min_x;
        p.y -= min_y;
        max_x = max_x.max(p.x + s.w);
        max_y = max_y.max(p.y + s.h);
    }
    (pts, Size { w: max_x, h: max_y })
}

/// Gap after each rank. Where a group begins or ends between two ranks, the gap also holds the
/// group's padding (and its title strip on the side that has it) plus `GROUP_GAP`, so edges and
/// label chips crossing into the group keep clear of its border.
fn rank_gaps(rank: &[usize], groups: &[Option<usize>], ranks: usize, dir: Direction) -> Vec<f32> {
    let mut span: HashMap<usize, (usize, usize)> = HashMap::new();
    for (i, g) in groups.iter().enumerate() {
        if let Some(g) = g {
            let e = span.entry(*g).or_insert((rank[i], rank[i]));
            e.0 = e.0.min(rank[i]);
            e.1 = e.1.max(rank[i]);
        }
    }
    let titled = GROUP_PAD + GROUP_TITLE;
    // The side of the group facing the earlier rank, and the side facing the later one.
    let (start_side, end_side) = match dir {
        Direction::Down => (titled, GROUP_PAD),
        Direction::Up => (GROUP_PAD, titled),
        Direction::Left | Direction::Right => (GROUP_PAD, GROUP_PAD),
    };
    (0..ranks)
        .map(|r| {
            let starts = span.values().any(|&(lo, _)| lo == r + 1);
            let ends = span.values().any(|&(_, hi)| hi == r);
            if !starts && !ends {
                return RANK_GAP;
            }
            let need = if starts { start_side } else { 0.0 }
                + if ends { end_side } else { 0.0 }
                + GROUP_GAP;
            need.max(RANK_GAP)
        })
        .collect()
}

/// Edges with cycles broken by reversing DFS back edges, duplicates removed.
fn dag_edges(m: usize, edges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let uniq: Vec<(usize, usize)> = edges.iter().copied().filter(|e| seen.insert(*e)).collect();
    let mut out: Vec<Vec<usize>> = vec![vec![]; m];
    let mut indeg = vec![0usize; m];
    for &(a, b) in &uniq {
        out[a].push(b);
        indeg[b] += 1;
    }
    let mut state = vec![0u8; m];
    let mut reversed: HashSet<(usize, usize)> = HashSet::new();
    let starts = (0..m)
        .filter(|&i| indeg[i] == 0)
        .chain((0..m).filter(|&i| indeg[i] > 0));
    for s in starts {
        if state[s] != 0 {
            continue;
        }
        state[s] = 1;
        let mut stack: Vec<(usize, usize)> = vec![(s, 0)];
        while let Some(&(v, i)) = stack.last() {
            let top = stack.len() - 1;
            if i < out[v].len() {
                stack[top].1 += 1;
                let w = out[v][i];
                match state[w] {
                    0 => {
                        state[w] = 1;
                        stack.push((w, 0));
                    }
                    1 => {
                        reversed.insert((v, w));
                    }
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    uniq.iter()
        .map(|&(a, b)| {
            if reversed.contains(&(a, b)) {
                (b, a)
            } else {
                (a, b)
            }
        })
        .filter(|e| seen.insert(*e))
        .collect()
}

/// Longest-path ranks over the cycle-free edges; sources are pulled down next to their children.
fn rank_nodes(m: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let dag = dag_edges(m, edges);
    let mut succ: Vec<Vec<usize>> = vec![vec![]; m];
    let mut indeg = vec![0usize; m];
    for &(a, b) in &dag {
        succ[a].push(b);
        indeg[b] += 1;
    }
    let sources: Vec<usize> = (0..m).filter(|&i| indeg[i] == 0).collect();
    let mut rank = vec![0usize; m];
    let mut queue: VecDeque<usize> = sources.iter().copied().collect();
    let mut left = indeg.clone();
    while let Some(v) = queue.pop_front() {
        for &w in &succ[v] {
            rank[w] = rank[w].max(rank[v] + 1);
            left[w] -= 1;
            if left[w] == 0 {
                queue.push_back(w);
            }
        }
    }
    for &s in &sources {
        if let Some(min) = succ[s].iter().map(|&w| rank[w]).min() {
            rank[s] = min.saturating_sub(1);
        }
    }
    let lo = rank.iter().copied().min().unwrap_or(0);
    rank.iter().map(|r| r - lo).collect()
}

/// Layers in DFS pre-order, which already keeps tree-like graphs free of crossings.
fn initial_order(
    lv: &[Lv],
    succs: &[Vec<usize>],
    preds: &[Vec<usize>],
    ranks: usize,
) -> Vec<Vec<usize>> {
    let total = lv.len();
    let mut seen_at = vec![usize::MAX; total];
    let mut counter = 0usize;
    let roots = (0..total).filter(|&v| preds[v].is_empty()).chain(0..total);
    for root in roots {
        let mut stack = vec![root];
        while let Some(v) = stack.pop() {
            if seen_at[v] != usize::MAX {
                continue;
            }
            seen_at[v] = counter;
            counter += 1;
            stack.extend(succs[v].iter().rev().copied());
        }
    }
    let mut layers: Vec<Vec<usize>> = vec![vec![]; ranks];
    let mut by_seen: Vec<usize> = (0..total).collect();
    by_seen.sort_by_key(|&v| seen_at[v]);
    for v in by_seen {
        layers[lv[v].rank].push(v);
    }
    layers
}

/// Barycentre sweeps, keeping the ordering with the fewest crossings.
fn order_layers(
    layers: &mut Vec<Vec<usize>>,
    lv: &[Lv],
    preds: &[Vec<usize>],
    succs: &[Vec<usize>],
    ledges: &[(usize, usize)],
) {
    let ranks = layers.len();
    if ranks < 2 {
        return;
    }
    let mut by_rank: Vec<Vec<(usize, usize)>> = vec![vec![]; ranks - 1];
    for &(u, v) in ledges {
        by_rank[lv[u].rank].push((u, v));
    }
    let mut pos = vec![0usize; lv.len()];
    let refresh = |layers: &[Vec<usize>], pos: &mut [usize]| {
        for l in layers {
            for (i, &v) in l.iter().enumerate() {
                pos[v] = i;
            }
        }
    };
    refresh(layers, &mut pos);
    for l in layers.iter_mut() {
        let keys: Vec<f32> = l.iter().map(|&v| pos[v] as f32).collect();
        sort_layer(l, &keys, lv, &mut pos);
    }
    let mut best = layers.clone();
    let mut best_c = crossings(layers, &by_rank, &pos);
    let mut stall = 0usize;
    for it in 0..MAX_SWEEPS {
        if it >= MIN_SWEEPS && (stall >= STALL_SWEEPS || best_c == 0) {
            break;
        }
        if it % 2 == 0 {
            for r in 1..ranks {
                reorder(layers, r, r - 1, preds, lv, &mut pos);
            }
        } else {
            for r in (0..ranks - 1).rev() {
                reorder(layers, r, r + 1, succs, lv, &mut pos);
            }
        }
        let c = crossings(layers, &by_rank, &pos);
        if c < best_c {
            best_c = c;
            best.clone_from(layers);
            stall = 0;
        } else {
            stall += 1;
        }
    }
    *layers = best;
}

fn reorder(
    layers: &mut [Vec<usize>],
    r: usize,
    other: usize,
    nbrs: &[Vec<usize>],
    lv: &[Lv],
    pos: &mut [usize],
) {
    let scale = layers[other].len() as f32 / layers[r].len().max(1) as f32;
    let keys: Vec<f32> = layers[r]
        .iter()
        .map(|&v| {
            if nbrs[v].is_empty() {
                (pos[v] as f32 + 0.5) * scale - 0.5
            } else {
                nbrs[v].iter().map(|&u| pos[u] as f32).sum::<f32>() / nbrs[v].len() as f32
            }
        })
        .collect();
    sort_layer(&mut layers[r], &keys, lv, pos);
}

/// Sort a layer by `keys` (parallel to the layer), but keep members of one group together,
/// the group sitting at the mean of its members' keys.
fn sort_layer(layer: &mut [usize], keys: &[f32], lv: &[Lv], pos: &mut [usize]) {
    // (cluster key, cluster id, own key, current position, node)
    let mut rows: Vec<(f32, usize, f32, usize, usize)> = layer
        .iter()
        .zip(keys)
        .map(|(&v, &k)| (k, 0, k, pos[v], v))
        .collect();
    let mut cluster: BTreeMap<usize, (f32, f32)> = BTreeMap::new();
    for row in &rows {
        if let Some(g) = lv[row.4].group {
            let e = cluster.entry(g).or_insert((0.0, 0.0));
            e.0 += row.2;
            e.1 += 1.0;
        }
    }
    for row in &mut rows {
        match lv[row.4].group {
            Some(g) => {
                let (sum, cnt) = cluster[&g];
                row.0 = sum / cnt;
                row.1 = g;
            }
            None => row.1 = usize::MAX / 2 + row.4,
        }
    }
    rows.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.total_cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });
    for (i, row) in rows.iter().enumerate() {
        layer[i] = row.4;
        pos[row.4] = i;
    }
}

/// Total edge crossings between adjacent layers.
fn crossings(layers: &[Vec<usize>], by_rank: &[Vec<(usize, usize)>], pos: &[usize]) -> usize {
    let mut total = 0;
    for (r, es) in by_rank.iter().enumerate() {
        let mut pairs: Vec<(usize, usize)> = es.iter().map(|&(u, v)| (pos[u], pos[v])).collect();
        pairs.sort_unstable();
        let mut tree = vec![0usize; layers[r + 1].len() + 1];
        for (seen, &(_, b)) in pairs.iter().enumerate() {
            // Earlier edges ending strictly right of `b` cross this one.
            let mut le = 0usize;
            let mut i = b + 1;
            while i > 0 {
                le += tree[i];
                i -= i & i.wrapping_neg();
            }
            total += seen - le;
            let mut i = b + 1;
            while i < tree.len() {
                tree[i] += 1;
                i += i & i.wrapping_neg();
            }
        }
    }
    total
}

/// Centre coordinate across the ranks for every layered node.
fn assign_secondary(
    layers: &[Vec<usize>],
    lv: &[Lv],
    preds: &[Vec<usize>],
    succs: &[Vec<usize>],
) -> Vec<f32> {
    let sep = |a: usize, b: usize| {
        let gap = if lv[a].dummy || lv[b].dummy {
            DUMMY_GAP
        } else {
            NODE_GAP
        };
        (lv[a].sec + lv[b].sec) / 2.0 + gap
    };
    let mut x = vec![0.0f32; lv.len()];
    for l in layers {
        let width: f32 = l.windows(2).map(|w| sep(w[0], w[1])).sum();
        let mut at = -width / 2.0;
        for (i, &v) in l.iter().enumerate() {
            if i > 0 {
                at += sep(l[i - 1], v);
            }
            x[v] = at;
        }
    }
    let ranks = layers.len();
    for pass in 0..PLACE_PASSES {
        let order: Vec<usize> = if pass % 2 == 0 {
            (1..ranks).collect()
        } else {
            (0..ranks.saturating_sub(1)).rev().collect()
        };
        let nbrs = if pass % 2 == 0 { preds } else { succs };
        for r in order {
            let l = &layers[r];
            let mut targets = Vec::with_capacity(l.len());
            let mut weights = Vec::with_capacity(l.len());
            for &v in l {
                if nbrs[v].is_empty() {
                    targets.push(x[v]);
                    weights.push(1.0);
                    continue;
                }
                let (mut sum, mut wsum) = (0.0f32, 0.0f32);
                for &u in &nbrs[v] {
                    let w = if lv[u].dummy && lv[v].dummy {
                        4.0
                    } else if lv[u].dummy || lv[v].dummy {
                        2.0
                    } else {
                        1.0
                    };
                    sum += w * x[u];
                    wsum += w;
                }
                targets.push(sum / wsum);
                weights.push(wsum);
            }
            let mut offsets = Vec::with_capacity(l.len());
            let mut o = 0.0f32;
            for (i, &v) in l.iter().enumerate() {
                if i > 0 {
                    o += sep(l[i - 1], v);
                }
                offsets.push(o);
            }
            let solved = ordered_fit(&targets, &weights, &offsets);
            for (&v, s) in l.iter().zip(solved) {
                x[v] = s;
            }
        }
    }
    // Left edge of the widest-left real node becomes 0.
    let min_left = (0..lv.len())
        .filter(|&v| !lv[v].dummy)
        .map(|v| x[v] - lv[v].sec / 2.0)
        .fold(f32::MAX, f32::min);
    x.iter().map(|v| v - min_left).collect()
}

/// Positions `p_i` closest (weighted least squares) to `targets` with `p_i - p_{i-1} >= offsets_i - offsets_{i-1}`.
/// Pool-adjacent-violators on `target - offset`.
fn ordered_fit(targets: &[f32], weights: &[f32], offsets: &[f32]) -> Vec<f32> {
    // Block: (weighted sum, weight, count)
    let mut blocks: Vec<(f32, f32, usize)> = vec![];
    for i in 0..targets.len() {
        blocks.push(((targets[i] - offsets[i]) * weights[i], weights[i], 1));
        while blocks.len() >= 2 {
            let (b, a) = (blocks[blocks.len() - 1], blocks[blocks.len() - 2]);
            if a.0 / a.1 <= b.0 / b.1 {
                break;
            }
            blocks.pop();
            let last = blocks.len() - 1;
            blocks[last] = (a.0 + b.0, a.1 + b.1, a.2 + b.2);
        }
    }
    let mut out = Vec::with_capacity(targets.len());
    for (sum, w, count) in blocks {
        for _ in 0..count {
            out.push(sum / w);
        }
    }
    out.iter().zip(offsets).map(|(y, o)| y + o).collect()
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::geometry::node_rect;
    use crate::model::Group;

    fn chain(n: usize, dir: Direction) -> Diagram {
        let mut d = Diagram::default();
        d.direction = dir;
        for i in 0..n {
            d.ensure_node(&format!("n{i}"), &format!("Step {i}"));
        }
        for i in 1..n {
            d.add_edge(&format!("n{}", i - 1), &format!("n{i}"));
        }
        d
    }

    fn overlaps(d: &Diagram) -> Vec<(String, String)> {
        let rs: Vec<Rect> = d.nodes.iter().map(|n| node_rect(d, n)).collect();
        let mut bad = vec![];
        for i in 0..rs.len() {
            for j in i + 1..rs.len() {
                let (a, b) = (rs[i], rs[j]);
                if a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h {
                    bad.push((d.nodes[i].id.clone(), d.nodes[j].id.clone()));
                }
            }
        }
        bad
    }

    fn cy(d: &Diagram, id: &str) -> f32 {
        node_rect(d, d.node(id).unwrap()).center().y
    }

    fn cx(d: &Diagram, id: &str) -> f32 {
        node_rect(d, d.node(id).unwrap()).center().x
    }

    #[test]
    fn empty_and_single() {
        let mut d = Diagram::default();
        auto_layout(&mut d);
        layout_unplaced(&mut d);
        assert!(d.nodes.is_empty());
        d.ensure_node("a", "Only");
        auto_layout(&mut d);
        assert!(d.nodes[0].pos.is_some());
    }

    #[test]
    fn every_direction_flows_the_right_way() {
        for (dir, check) in [
            (Direction::Down, 0),
            (Direction::Up, 1),
            (Direction::Right, 2),
            (Direction::Left, 3),
        ] {
            let mut d = chain(4, dir);
            auto_layout(&mut d);
            for i in 1..4 {
                let (a, b) = (format!("n{}", i - 1), format!("n{i}"));
                match check {
                    0 => assert!(cy(&d, &b) > cy(&d, &a)),
                    1 => assert!(cy(&d, &b) < cy(&d, &a)),
                    2 => assert!(cx(&d, &b) > cx(&d, &a)),
                    _ => assert!(cx(&d, &b) < cx(&d, &a)),
                }
            }
            assert!(overlaps(&d).is_empty());
        }
    }

    #[test]
    fn rank_gap_is_respected() {
        let mut d = chain(3, Direction::Down);
        auto_layout(&mut d);
        let a = node_rect(&d, &d.nodes[0]);
        let b = node_rect(&d, &d.nodes[1]);
        assert!((b.y - (a.y + a.h) - RANK_GAP).abs() < 0.01);
        let mut d = chain(3, Direction::Right);
        auto_layout(&mut d);
        let a = node_rect(&d, &d.nodes[0]);
        let b = node_rect(&d, &d.nodes[1]);
        assert!((b.x - (a.x + a.w) - RANK_GAP).abs() < 0.01);
    }

    #[test]
    fn siblings_are_spaced_and_parent_centred() {
        let mut d = Diagram::default();
        for id in ["p", "a", "b", "c"] {
            d.ensure_node(id, id);
        }
        for c in ["a", "b", "c"] {
            d.add_edge("p", c);
        }
        auto_layout(&mut d);
        assert!(overlaps(&d).is_empty());
        let ra = node_rect(&d, d.node("a").unwrap());
        let rb = node_rect(&d, d.node("b").unwrap());
        assert!(rb.x - (ra.x + ra.w) >= NODE_GAP - 0.01);
        assert!((cx(&d, "p") - cx(&d, "b")).abs() < 1.0);
    }

    #[test]
    fn cycles_and_self_loops_terminate() {
        let mut d = chain(3, Direction::Down);
        d.add_edge("n2", "n0");
        d.add_edge("n1", "n1");
        d.add_edge("n1", "n0");
        auto_layout(&mut d);
        assert!(overlaps(&d).is_empty());
        assert!(cy(&d, "n1") > cy(&d, "n0") && cy(&d, "n2") > cy(&d, "n1"));
    }

    #[test]
    fn ordering_removes_a_simple_crossing() {
        let mut d = Diagram::default();
        for id in ["a", "b", "c", "d"] {
            d.ensure_node(id, id);
        }
        d.add_edge("a", "d");
        d.add_edge("b", "c");
        auto_layout(&mut d);
        // a is left of b exactly when d is left of c.
        assert_eq!(cx(&d, "a") < cx(&d, "b"), cx(&d, "d") < cx(&d, "c"));
    }

    #[test]
    fn crossing_counter_counts() {
        let layers = vec![vec![0, 1], vec![2, 3]];
        let by_rank = vec![vec![(0, 3), (1, 2)]];
        let mut pos = vec![0; 4];
        for l in &layers {
            for (i, &v) in l.iter().enumerate() {
                pos[v] = i;
            }
        }
        assert_eq!(crossings(&layers, &by_rank, &pos), 1);
        let by_rank = vec![vec![(0, 2), (1, 3), (0, 3)]];
        assert_eq!(crossings(&layers, &by_rank, &pos), 0);
    }

    #[test]
    fn ordered_fit_respects_order_and_gaps() {
        let r = ordered_fit(&[10.0, 10.0, 10.0], &[1.0; 3], &[0.0, 5.0, 10.0]);
        assert!((r[1] - r[0] - 5.0).abs() < 1e-4 && (r[2] - r[1] - 5.0).abs() < 1e-4);
        assert!((r[1] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn disconnected_components_sit_side_by_side() {
        let mut d = chain(3, Direction::Down);
        for i in 0..3 {
            d.ensure_node(&format!("m{i}"), "Other");
        }
        d.add_edge("m0", "m1");
        d.ensure_node("solo", "Alone");
        auto_layout(&mut d);
        assert!(overlaps(&d).is_empty());
        assert!(cx(&d, "m0") > cx(&d, "n0"));
        assert!(cx(&d, "solo") > cx(&d, "m0"));
    }

    #[test]
    fn long_edges_reserve_room() {
        let mut d = chain(4, Direction::Down);
        d.add_edge("n0", "n3");
        auto_layout(&mut d);
        assert!(overlaps(&d).is_empty());
    }

    #[test]
    fn group_members_stay_adjacent_within_a_rank() {
        let mut d = Diagram::default();
        d.groups.push(Group {
            id: "g".into(),
            label: "G".into(),
            role: crate::brand::Role::Neutral,
        });
        d.ensure_node("root", "root");
        for id in ["a", "b", "c", "e"] {
            d.ensure_node(id, id);
            d.add_edge("root", id);
        }
        d.node_mut("a").unwrap().group = Some("g".into());
        d.node_mut("c").unwrap().group = Some("g".into());
        auto_layout(&mut d);
        let (a, c) = (cx(&d, "a"), cx(&d, "c"));
        let (lo, hi) = (a.min(c), a.max(c));
        for id in ["b", "e"] {
            let x = cx(&d, id);
            assert!(x < lo || x > hi, "{id} sits inside the group");
        }
    }

    #[test]
    fn deterministic() {
        let build = || {
            let mut d = chain(6, Direction::Right);
            d.add_edge("n0", "n3");
            d.add_edge("n4", "n1");
            d.add_edge("n2", "n5");
            d.ensure_node("x", "x");
            d
        };
        let (mut a, mut b) = (build(), build());
        auto_layout(&mut a);
        auto_layout(&mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn auto_layout_overwrites_everything() {
        let mut d = chain(2, Direction::Down);
        d.nodes[0].pos = Some(Point::new(900.0, 900.0));
        auto_layout(&mut d);
        assert!(d.nodes[0].pos.unwrap().x < 500.0);
    }

    #[test]
    fn unplaced_nodes_land_near_placed_neighbours_without_overlap() {
        let mut d = chain(3, Direction::Down);
        auto_layout(&mut d);
        let before: Vec<_> = d.nodes.iter().map(|n| n.pos).collect();
        // Hand-move the diagram away, then add nodes.
        for n in &mut d.nodes {
            let p = n.pos.unwrap();
            n.pos = Some(Point::new(p.x + 1000.0, p.y + 400.0));
        }
        let moved: Vec<_> = d.nodes.iter().map(|n| n.pos).collect();
        d.ensure_node("new", "New step");
        d.add_edge("n2", "new");
        d.ensure_node("sib", "Sibling");
        d.add_edge("n2", "sib");
        d.ensure_node("far", "Unconnected");
        layout_unplaced(&mut d);
        assert_ne!(before, moved);
        for (n, p) in d.nodes.iter().zip(&moved) {
            assert_eq!(&n.pos, p);
        }
        assert!(overlaps(&d).is_empty());
        assert!(cy(&d, "new") > cy(&d, "n2") && cy(&d, "sib") > cy(&d, "n2"));
        assert!((cx(&d, "new") - cx(&d, "n2")).abs() < 400.0);
        let n2 = node_rect(&d, d.node("n2").unwrap());
        assert!(
            node_rect(&d, d.node("far").unwrap()).x >= n2.x + n2.w,
            "isolated node goes beside the diagram"
        );
    }

    #[test]
    fn unplaced_with_nothing_placed_matches_auto_layout() {
        let mut a = chain(4, Direction::Down);
        let mut b = a.clone();
        auto_layout(&mut a);
        layout_unplaced(&mut b);
        assert_eq!(a, b);
    }

    #[test]
    fn five_hundred_nodes_are_fast() {
        let mut d = Diagram::default();
        let mut seed = 12345u64;
        let mut next = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for i in 0..500 {
            d.ensure_node(&format!("n{i}"), &format!("Node {i}"));
        }
        for i in 1..500 {
            let parent = next() % i;
            d.add_edge(&format!("n{parent}"), &format!("n{i}"));
            if i % 3 == 0 {
                let other = next() % i;
                d.add_edge(&format!("n{other}"), &format!("n{i}"));
            }
            if i % 50 == 0 {
                d.add_edge(&format!("n{i}"), "n0");
            }
        }
        let t = std::time::Instant::now();
        auto_layout(&mut d);
        let ms = t.elapsed().as_millis();
        let limit = if cfg!(debug_assertions) { 5000 } else { 200 };
        assert!(ms < limit, "{ms}ms");
        assert!(overlaps(&d).is_empty());
    }
}
