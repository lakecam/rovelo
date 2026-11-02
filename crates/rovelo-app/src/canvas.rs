//! The infinite canvas: painting the diagram and every pointer interaction on it.
use crate::app::{Cmd, RoveloApp};
use crate::ops;
use crate::quick::{self, Side};
use crate::templates::Template;
use crate::theme::{self, Tokens, rgb};
use crate::view::{GRID, View, from_core, marquee_hits, pt, snap, snapped_delta};
use crate::{chrome_icons, shapes};
use egui::{
    Color32, CursorIcon, FontId, Key, PointerButton, Pos2, Rect, Sense, Shape as EShape, Stroke,
    TextEdit, Ui, UiBuilder, Vec2, pos2, vec2,
};
use rovelo_core::brand::{BrandKit, Role};
use rovelo_core::geometry::{self, EdgePath};
use rovelo_core::model::{Arrow, Diagram, Id, LineStyle, Point};

/// Height kept clear for the floating pill rows (12 + 44 + 12).
const PILL_ROW: f32 = 68.0;
/// Space between the fitted diagram and the free area's edges.
const FIT_MARGIN: f32 = 24.0;
const EDGE_HIT: f32 = 7.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    AddNode,
    Connect,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EditTarget {
    Node(Id),
    Edge(Id),
}

#[derive(Clone, Debug)]
pub struct Editing {
    pub target: EditTarget,
    pub fresh: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
enum Hit {
    #[default]
    None,
    Handle(Id, Side),
    Node(Id),
    Edge(Id),
    Group(Id),
}

#[derive(Default)]
enum Drag {
    #[default]
    None,
    Pan,
    Move {
        anchor: Id,
        starts: Vec<(Id, Pos2)>,
        total: Vec2,
    },
    Marquee {
        start: Pos2,
        additive: bool,
        base: Vec<Id>,
    },
    Connect {
        from: Id,
        /// The handle the drag left from; `None` for the Connect tool's drag from a node body.
        side: Option<Side>,
        start_screen: Pos2,
    },
    /// Add-step tool drag drawing the new step's box.
    Create {
        start: Pos2,
        start_screen: Pos2,
        cancelled: bool,
    },
}

pub struct CanvasState {
    pub view: View,
    pub origin: Pos2,
    pub size: Vec2,
    pub fit_request: bool,
    /// Keep re-fitting while the panels settle, until the user moves the view.
    auto_fit: bool,
    fit_size: Vec2,
    fit_frames: u8,
    fit_right: f32,
    pub zoom_request: Option<f32>,
    pub center_request: Option<Id>,
    pub editing: Option<Editing>,
    /// Node a hovered task row points at.
    pub task_highlight: Option<Id>,
    pub tool: Tool,
    /// A just-made step to pan into view, the least it takes, zoom kept.
    pub uncover: Option<Id>,
    /// First node picked by click-click connecting.
    connect_from: Option<Id>,
    drag: Drag,
    hover: Hit,
    ctx_world: Pos2,
    started: bool,
}

impl Default for CanvasState {
    fn default() -> Self {
        CanvasState {
            view: View::default(),
            origin: Pos2::ZERO,
            size: vec2(800.0, 600.0),
            fit_request: false,
            auto_fit: false,
            fit_size: Vec2::ZERO,
            fit_frames: 0,
            fit_right: 0.0,
            zoom_request: None,
            center_request: None,
            editing: None,
            task_highlight: None,
            tool: Tool::Select,
            uncover: None,
            connect_from: None,
            drag: Drag::None,
            hover: Hit::None,
            ctx_world: Pos2::ZERO,
            started: false,
        }
    }
}

impl CanvasState {
    pub fn reset_for_new_document(&mut self) {
        self.drag = Drag::None;
        self.editing = None;
        self.hover = Hit::None;
        self.center_request = None;
        self.view = View::default();
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.connect_from = None;
    }

    pub fn start_edit(&mut self, id: &str, d: &Diagram) {
        if d.node(id).is_some() {
            self.editing = Some(Editing {
                target: EditTarget::Node(id.to_string()),
                fresh: true,
            });
        }
    }
}

struct Geo {
    nodes: Vec<Rect>,
    edges: Vec<Option<EdgePath>>,
    groups: Vec<Option<Rect>>,
}

fn compute_geo(d: &Diagram) -> Geo {
    Geo {
        nodes: d
            .nodes
            .iter()
            .map(|n| from_core(geometry::node_rect(d, n)))
            .collect(),
        edges: d
            .edges
            .iter()
            .map(|e| {
                (d.node(&e.from).is_some() && d.node(&e.to).is_some())
                    .then(|| geometry::edge_path(d, e))
            })
            .collect(),
        groups: d
            .groups
            .iter()
            .map(|g| geometry::group_rect(d, &g.id).map(from_core))
            .collect(),
    }
}

fn world_bounds(d: &Diagram, geo: &Geo) -> Option<Rect> {
    let mut b: Option<Rect> = None;
    let mut add = |r: Rect| b = Some(b.map_or(r, |x| x.union(r)));
    geo.nodes.iter().for_each(|r| add(*r));
    geo.groups.iter().flatten().for_each(|r| add(*r));
    for p in geo.edges.iter().flatten() {
        for q in &p.points {
            add(Rect::from_min_max(pt(*q), pt(*q)));
        }
    }
    let _ = d;
    b.filter(|r| r.width().is_finite() && r.height().is_finite())
}

fn core_rect(r: Rect) -> rovelo_core::model::Rect {
    rovelo_core::model::Rect {
        x: r.min.x,
        y: r.min.y,
        w: r.width(),
        h: r.height(),
    }
}

fn hit_test(d: &Diagram, geo: &Geo, world: Pos2, zoom: f32, handle_nodes: &[Id]) -> Hit {
    for id in handle_nodes {
        if let Some(i) = d.nodes.iter().position(|n| &n.id == id)
            && let Some(side) =
                quick::pick_handle(core_rect(geo.nodes[i]), Point::new(world.x, world.y), zoom)
        {
            return Hit::Handle(id.clone(), side);
        }
    }
    for (i, n) in d.nodes.iter().enumerate().rev() {
        if geo.nodes[i].contains(world) {
            return Hit::Node(n.id.clone());
        }
    }
    for (i, e) in d.edges.iter().enumerate().rev() {
        if let Some(p) = &geo.edges[i] {
            if geometry::distance_to_path(p, Point::new(world.x, world.y)) * zoom <= EDGE_HIT {
                return Hit::Edge(e.id.clone());
            }
            if !e.label.trim().is_empty()
                && pill_size(&e.label, d.brand.font_size)
                    .map(|s| Rect::from_center_size(pt(p.label_at), s))
                    .is_some_and(|r| r.contains(world))
            {
                return Hit::Edge(e.id.clone());
            }
        }
    }
    for (i, g) in d.groups.iter().enumerate().rev() {
        if let Some(r) = geo.groups[i] {
            let title = Rect::from_min_size(r.min, vec2(r.width(), geometry::GROUP_TITLE));
            let edge_tol = 6.0 / zoom;
            let on_border =
                r.expand(edge_tol).contains(world) && !r.shrink(edge_tol).contains(world);
            if title.contains(world) || on_border {
                return Hit::Group(g.id.clone());
            }
        }
    }
    Hit::None
}

/// Edge label pill size in world units, as the SVG exporter sizes it.
fn pill_size(label: &str, font_size: f32) -> Option<Vec2> {
    let fs = font_size.clamp(6.0, 96.0) * 0.86;
    Some(vec2(
        geometry::text_width(label, fs) + 16.0,
        geometry::label_lines(label).len() as f32 * fs * 1.25 + 8.0,
    ))
}

fn font_px(brand: &BrandKit, zoom: f32, scale: f32) -> f32 {
    (brand.font_size * scale * zoom).clamp(0.0, 120.0)
}

pub fn show(app: &mut RoveloApp, ui: &mut Ui) {
    let left_inset = if app.panel_open {
        crate::panels::side::PANEL_W + 12.0
    } else {
        0.0
    };
    let RoveloApp {
        diagram,
        sel,
        canvas: cv,
        cmds,
        ..
    } = app;
    let rect = ui.available_rect_before_wrap();
    let resp = ui.allocate_rect(rect, Sense::click_and_drag());
    cv.origin = rect.min;
    cv.size = rect.size();
    cv.view.sanitize();
    if !cv.started {
        cv.started = true;
    }
    let tokens = Tokens::of(ui);
    let accent = tokens.accent;
    let typing = ui.ctx().egui_wants_keyboard_input();
    let origin = rect.min;

    // ---- view requests -------------------------------------------------
    let geo = compute_geo(diagram);
    // The part of the window not covered by the floating chrome: the left panel (when open),
    // the inspector (while something is selected) and the pill rows top and bottom.
    // The inspector only counts at an explicit fit, so selecting never moves the view.
    if cv.fit_request {
        cv.fit_right = if sel.is_empty() {
            0.0
        } else {
            crate::panels::inspector::PANEL_W + 12.0
        };
    }
    let right_inset = cv.fit_right;
    let free = Rect::from_min_max(
        rect.min + vec2(left_inset, PILL_ROW),
        rect.max - vec2(right_inset, PILL_ROW),
    );
    let free_off = free.min - rect.min;
    let free_size = vec2(free.width().max(120.0), free.height().max(120.0));
    // Re-fit while the window and panels settle; the inspector opening never moves the view.
    let basis = vec2(rect.width() - left_inset, rect.height());
    let refit = cv.auto_fit && (cv.fit_frames > 0 || (basis - cv.fit_size).abs().max_elem() > 1.0);
    if cv.fit_request || refit {
        if cv.fit_request {
            cv.fit_frames = 12;
        }
        cv.fit_frames = cv.fit_frames.saturating_sub(1);
        cv.auto_fit = true;
        cv.fit_size = basis;
        cv.fit_request = false;
        if let Some(b) = world_bounds(diagram, &geo) {
            cv.view = View::fit(b, free_size, FIT_MARGIN);
        } else {
            cv.view = View::default();
        }
        cv.view.pan += free_off;
    }
    if let Some(f) = cv.zoom_request.take() {
        cv.auto_fit = false;
        cv.view.zoom_about(origin, free.center(), f);
    }
    if let Some(id) = cv.uncover.take()
        && let Some(i) = diagram.nodes.iter().position(|n| n.id == id)
    {
        let right = if sel.is_empty() {
            0.0
        } else {
            crate::panels::inspector::PANEL_W + 12.0
        };
        let clear = Rect::from_min_max(
            rect.min + vec2(left_inset, PILL_ROW),
            rect.max - vec2(right, PILL_ROW),
        );
        let pan = quick::pan_to_uncover(cv.view.rect_to_screen(origin, geo.nodes[i]), clear);
        if pan != Vec2::ZERO {
            cv.auto_fit = false;
            cv.view.pan += pan;
        }
    }
    if let Some(id) = cv.center_request.take() {
        cv.auto_fit = false;
        if let Some(i) = diagram.nodes.iter().position(|n| n.id == id) {
            cv.view.center_on(geo.nodes[i].center(), free_size);
            cv.view.pan += free_off;
        }
    }

    // ---- wheel / pinch -------------------------------------------------
    // Wheel input is ignored while a fresh fit settles, so stray momentum cannot shove it aside.
    if resp.hovered() && cv.fit_frames == 0 {
        let (scroll, zoom_d) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta()));
        let ptr = ui.input(|i| i.pointer.hover_pos()).unwrap_or(rect.center());
        if (zoom_d - 1.0).abs() > 1e-4 || scroll.length() > 0.5 {
            cv.auto_fit = false;
        }
        if (zoom_d - 1.0).abs() > 1e-4 {
            cv.view.zoom_about(origin, ptr, zoom_d);
        } else if scroll != Vec2::ZERO {
            cv.view.pan += scroll;
        }
    }

    let zoom = cv.view.zoom;
    let view = cv.view;
    let space = ui.input(|i| i.key_down(Key::Space)) && !typing;
    let mods = ui.input(|i| i.modifiers);
    let ptr_screen = ui.input(|i| i.pointer.hover_pos());
    let ptr_world = ptr_screen.map(|p| view.to_world(origin, p));

    let handle_nodes: Vec<Id> = {
        let mut v = Vec::new();
        if sel.nodes.len() == 1 {
            v.push(sel.nodes[0].clone());
        }
        if let Hit::Node(id) | Hit::Handle(id, _) = &cv.hover
            && !v.contains(id)
        {
            v.push(id.clone());
        }
        v
    };

    // hover (only meaningful while no gesture runs)
    if matches!(cv.drag, Drag::None) {
        cv.hover = match (ptr_world, resp.hovered()) {
            (Some(w), true) => hit_test(diagram, &geo, w, zoom, &handle_nodes),
            _ => Hit::None,
        };
    }

    // ---- drag start ----------------------------------------------------
    if resp.drag_started() && cv.editing.is_none() {
        cv.auto_fit = false;
        let press = ui
            .input(|i| i.pointer.press_origin())
            .or(ptr_screen)
            .unwrap_or(rect.center());
        let w = view.to_world(origin, press);
        let middle = ui.input(|i| i.pointer.button_down(PointerButton::Middle));
        if middle || space {
            cv.drag = Drag::Pan;
        } else {
            match hit_test(diagram, &geo, w, zoom, &handle_nodes) {
                Hit::Handle(id, side) => {
                    cv.drag = Drag::Connect {
                        from: id,
                        side: Some(side),
                        start_screen: press,
                    }
                }
                Hit::Node(id) if cv.tool == Tool::Connect => {
                    cv.drag = Drag::Connect {
                        from: id,
                        side: None,
                        start_screen: press,
                    }
                }
                _ if cv.tool == Tool::AddNode => {
                    cv.drag = Drag::Create {
                        start: w,
                        start_screen: press,
                        cancelled: false,
                    }
                }
                Hit::Node(id) => {
                    if !sel.has_node(&id) {
                        if mods.shift {
                            sel.toggle_node(&id);
                        } else {
                            sel.only_node(&id);
                        }
                    }
                    sel.edges.clear();
                    sel.groups.clear();
                    let starts = sel
                        .nodes
                        .iter()
                        .filter_map(|n| {
                            diagram
                                .nodes
                                .iter()
                                .position(|x| &x.id == n)
                                .map(|i| (n.clone(), geo.nodes[i].min))
                        })
                        .collect();
                    cv.drag = Drag::Move {
                        anchor: id,
                        starts,
                        total: Vec2::ZERO,
                    };
                }
                Hit::Group(gid) => {
                    sel.only_group(&gid);
                    let members: Vec<(Id, Pos2)> = diagram
                        .nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| n.group.as_deref() == Some(gid.as_str()))
                        .map(|(i, n)| (n.id.clone(), geo.nodes[i].min))
                        .collect();
                    if let Some((anchor, _)) = members.first().cloned() {
                        cv.drag = Drag::Move {
                            anchor,
                            starts: members,
                            total: Vec2::ZERO,
                        };
                    }
                }
                Hit::Edge(_) | Hit::None => {
                    let base = if mods.shift {
                        sel.nodes.clone()
                    } else {
                        Vec::new()
                    };
                    if !mods.shift {
                        sel.clear();
                    }
                    cv.drag = Drag::Marquee {
                        start: w,
                        additive: mods.shift,
                        base,
                    };
                }
            }
        }
    }

    // ---- drag update ---------------------------------------------------
    if resp.dragged() {
        let delta = resp.drag_delta();
        match &mut cv.drag {
            Drag::Pan => cv.view.pan += delta,
            Drag::Move {
                anchor,
                starts,
                total,
            } => {
                *total += delta / zoom;
                let anchor_start = starts
                    .iter()
                    .find(|(id, _)| id == anchor)
                    .map(|(_, p)| *p)
                    .unwrap_or(Pos2::ZERO);
                let d = snapped_delta(anchor_start, *total, GRID, !mods.alt);
                for (id, p) in starts.iter() {
                    if let Some(n) = diagram.node_mut(id) {
                        n.pos = Some(Point::new(p.x + d.x, p.y + d.y));
                    }
                }
            }
            Drag::Marquee {
                start,
                additive,
                base,
            } => {
                if let Some(w) = ptr_world {
                    let m = Rect::from_two_pos(*start, w);
                    let items: Vec<(String, Rect)> = diagram
                        .nodes
                        .iter()
                        .enumerate()
                        .map(|(i, n)| (n.id.clone(), geo.nodes[i]))
                        .collect();
                    let mut hits = marquee_hits(&items, m);
                    if *additive {
                        for b in base.iter() {
                            if !hits.contains(b) {
                                hits.push(b.clone());
                            }
                        }
                    }
                    sel.nodes = hits;
                }
            }
            Drag::Create { cancelled, .. } => {
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    *cancelled = true;
                }
            }
            Drag::Connect { .. } | Drag::None => {}
        }
    }

    // ---- drag end ------------------------------------------------------
    if resp.drag_stopped() {
        let drag = std::mem::take(&mut cv.drag);
        let moved_from = |s: Pos2| ptr_screen.map_or(0.0, |p| (p - s).length());
        match drag {
            Drag::Connect {
                from,
                side,
                start_screen,
            } => {
                let target = ptr_world.and_then(|w| match hit_test(diagram, &geo, w, zoom, &[]) {
                    Hit::Node(id) => Some(id),
                    _ => None,
                });
                match (side, target) {
                    (Some(side), _) if moved_from(start_screen) < quick::CLICK_PX => {
                        quick_create(diagram, sel, cv, &from, side, None);
                    }
                    (_, Some(to)) if to != from => {
                        if let Some(e) = ops::connect(diagram, &from, &to) {
                            sel.only_edge(&e);
                        }
                        cv.set_tool(Tool::Select);
                    }
                    // Released back on the source: nothing.
                    (_, Some(_)) => {}
                    (side, None) => {
                        if let Some(w) = ptr_world {
                            quick_create(
                                diagram,
                                sel,
                                cv,
                                &from,
                                side.unwrap_or(Side::Right),
                                Some(w),
                            );
                        }
                    }
                }
            }
            Drag::Create {
                start,
                start_screen,
                cancelled: false,
            } => {
                if moved_from(start_screen) < quick::CLICK_PX {
                    place_step(diagram, sel, cv, start, None);
                } else if let Some(w) = ptr_world {
                    let b = quick::drag_box(
                        Point::new(start.x, start.y),
                        Point::new(w.x, w.y),
                        mods.shift,
                    );
                    let snapped = rovelo_core::model::Rect {
                        x: snap(b.x, GRID),
                        y: snap(b.y, GRID),
                        w: snap(b.w, GRID).max(GRID),
                        h: snap(b.h, GRID).max(GRID),
                    };
                    place_step(diagram, sel, cv, start, Some(snapped));
                }
            }
            _ => {}
        }
    }

    // ---- click ---------------------------------------------------------
    if resp.clicked_by(PointerButton::Primary)
        && let Some(w) = resp
            .interact_pointer_pos()
            .map(|p| view.to_world(origin, p))
    {
        let hit = hit_test(diagram, &geo, w, zoom, &handle_nodes);
        match (cv.tool, hit) {
            (Tool::AddNode, _) => place_step(diagram, sel, cv, w, None),
            (Tool::Select, Hit::Handle(id, side)) => {
                quick_create(diagram, sel, cv, &id, side, None)
            }
            (Tool::Connect, Hit::Node(id) | Hit::Handle(id, _)) => match cv.connect_from.take() {
                Some(from) if from != id => {
                    if let Some(e) = ops::connect(diagram, &from, &id) {
                        sel.only_edge(&e);
                    }
                }
                Some(_) => {}
                None => cv.connect_from = Some(id),
            },
            (Tool::Connect, _) => cv.connect_from = None,
            (Tool::Select, hit) => match hit {
                Hit::Node(id) | Hit::Handle(id, _) => {
                    if mods.shift {
                        sel.toggle_node(&id);
                    } else {
                        sel.only_node(&id);
                    }
                }
                Hit::Edge(id) => {
                    if mods.shift {
                        sel.toggle_edge(&id);
                    } else {
                        sel.only_edge(&id);
                    }
                }
                Hit::Group(id) => sel.only_group(&id),
                Hit::None => {
                    if !mods.shift {
                        sel.clear();
                    }
                }
            },
        }
    }
    if resp.double_clicked_by(PointerButton::Primary)
        && let Some(w) = resp
            .interact_pointer_pos()
            .map(|p| view.to_world(origin, p))
    {
        match hit_test(diagram, &geo, w, zoom, &handle_nodes) {
            Hit::Node(id) => {
                cv.start_edit(&id, diagram);
            }
            // The first click of the pair already made a step from the handle.
            Hit::Handle(..) => {}
            Hit::Edge(id) => {
                if diagram.edges.iter().any(|e| e.id == id) {
                    cv.editing = Some(Editing {
                        target: EditTarget::Edge(id.clone()),
                        fresh: true,
                    });
                }
            }
            Hit::Group(_) => {}
            Hit::None => place_step(diagram, sel, cv, w, None),
        }
    }

    // ---- context menu --------------------------------------------------
    if resp.secondary_clicked()
        && let Some(w) = ptr_world
    {
        cv.ctx_world = w;
        match hit_test(diagram, &geo, w, zoom, &[]) {
            Hit::Node(id) if !sel.has_node(&id) => {
                sel.only_node(&id);
            }
            Hit::Edge(id) => sel.only_edge(&id),
            Hit::Group(id) => sel.only_group(&id),
            _ => {}
        }
    }
    let ctx_world = cv.ctx_world;
    resp.context_menu(|ui| {
        if ui.button("Add node here").clicked() {
            let id = ops::add_node_at(
                diagram,
                Point::new(snap(ctx_world.x, GRID), snap(ctx_world.y, GRID)),
                "New step",
            );
            sel.only_node(&id);
            cv.start_edit(&id, diagram);
            ui.close();
        }
        if !sel.nodes.is_empty() {
            if sel.nodes.len() == 1 && ui.button("Edit label").clicked() {
                let id = sel.nodes[0].clone();
                cv.start_edit(&id, diagram);
                ui.close();
            }
            if ui.button("Duplicate    \u{2318}D").clicked() {
                cmds.push(Cmd::Duplicate);
                ui.close();
            }
            if ui.button("Group    \u{2318}G").clicked() {
                cmds.push(Cmd::Group);
                ui.close();
            }
        }
        if !sel.is_empty() && ui.button("Delete").clicked() {
            cmds.push(Cmd::DeleteSelection);
            ui.close();
        }
        ui.separator();
        if ui.button("Zoom to fit").clicked() {
            cmds.push(Cmd::Fit);
            ui.close();
        }
    });

    // ---- cursor --------------------------------------------------------
    if resp.hovered() || !matches!(cv.drag, Drag::None) {
        let icon = match (&cv.drag, &cv.hover) {
            (Drag::Pan, _) => Some(CursorIcon::Grabbing),
            (Drag::Move { .. }, _) => Some(CursorIcon::Grabbing),
            (Drag::Connect { .. }, _) => Some(CursorIcon::Crosshair),
            (_, _) if space => Some(CursorIcon::Grab),
            (_, _) if cv.tool != Tool::Select => Some(CursorIcon::Crosshair),
            (_, Hit::Handle(..)) => Some(CursorIcon::PointingHand),
            (_, Hit::Node(_)) | (_, Hit::Edge(_)) | (_, Hit::Group(_)) => {
                Some(CursorIcon::PointingHand)
            }
            _ => None,
        };
        if let Some(i) = icon {
            ui.ctx().set_cursor_icon(i);
        }
    }

    // ---- paint ---------------------------------------------------------
    let geo = compute_geo(diagram);
    let painter = ui.painter_at(rect);
    let brand = diagram.brand.clone();
    painter.rect_filled(rect, 0, tokens.paper);
    paint_grid(&painter, rect, &view, tokens.dot);
    let clip = rect.expand(80.0);
    let cards = rovelo_core::card::uses_cards(&brand);
    let dark = tokens.dark;
    let editing_node = match &cv.editing {
        Some(Editing {
            target: EditTarget::Node(id),
            ..
        }) => Some(id.clone()),
        _ => None,
    };

    // groups
    for (i, g) in diagram.groups.iter().enumerate() {
        let Some(r) = geo.groups[i] else { continue };
        let sr = view.rect_to_screen(origin, r);
        if !sr.intersects(clip) {
            continue;
        }
        let selected = sel.groups.contains(&g.id);
        if cards {
            crate::cards::paint_glass_group(
                &painter, &brand, g.role, &g.label, sr, zoom, dark, selected, accent,
            );
            continue;
        }
        let col = brand.palette.get(g.role);
        let rad = ((brand.corner_radius + 4.0) * zoom).min(sr.width().min(sr.height()) / 2.0);
        painter.rect(
            sr,
            rad,
            rgb(brand.background.mix(col, 0.09)),
            Stroke::new(
                if selected { 2.0 } else { 1.0 },
                if selected {
                    accent
                } else {
                    rgb(col.mix(brand.background, 0.5))
                },
            ),
            egui::StrokeKind::Inside,
        );
        let px = font_px(&brand, zoom, 0.86);
        if px >= 5.0 {
            let ink = rgb(col.mix(brand.text, 0.5));
            let galley = painter.layout_no_wrap(g.label.clone(), FontId::proportional(px), ink);
            painter.galley(
                sr.min + vec2(14.0 * zoom, 19.0 * zoom - galley.size().y * 0.75),
                galley,
                ink,
            );
        }
    }

    // edges
    for (i, e) in diagram.edges.iter().enumerate() {
        let Some(path) = &geo.edges[i] else { continue };
        if path.points.len() < 2 {
            continue;
        }
        let selected = sel.edges.contains(&e.id);
        if cards {
            crate::cards::paint_card_edge(
                &painter, e, path, &brand, &view, origin, dark, selected, accent,
            );
        } else {
            paint_edge(&painter, e, path, &brand, &view, origin, selected, accent);
        }
    }

    // nodes
    for (i, n) in diagram.nodes.iter().enumerate() {
        let sr = view.rect_to_screen(origin, geo.nodes[i]);
        if !sr.intersects(clip) || !sr.min.is_finite() || !sr.max.is_finite() {
            continue;
        }
        let tasks: Vec<_> = diagram.tasks_for(&n.id).collect();
        let badge =
            (!tasks.is_empty()).then(|| (tasks.iter().filter(|t| t.done).count(), tasks.len()));
        let selected = sel.has_node(&n.id);
        let hovered = matches!(&cv.hover, Hit::Node(id) | Hit::Handle(id, _) if id == &n.id);
        let highlight = cv.task_highlight.as_deref() == Some(n.id.as_str());
        let editing = editing_node.as_deref() == Some(n.id.as_str());
        if cards {
            // Ports only while connecting (on the source and the hovered target); the plus
            // handles take their place on a selected or hovered node, as in the original.
            let source = match &cv.drag {
                Drag::Connect { from, .. } => Some(from),
                _ => cv.connect_from.as_ref(),
            };
            let ports = source.is_some_and(|f| f == &n.id || hovered);
            let st = crate::cards::CardState {
                selected,
                highlight,
                editing,
                ports,
            };
            crate::cards::paint_card(&painter, diagram, n, sr, zoom, dark, &st, accent);
        } else {
            paint_node(
                &painter,
                diagram,
                dark,
                n,
                geo.nodes[i],
                sr,
                &brand,
                zoom,
                selected,
                highlight,
                badge,
                editing,
                accent,
            );
        }
        if (selected && sel.nodes.len() == 1 || hovered)
            && matches!(cv.drag, Drag::None)
            && cv.tool == Tool::Select
        {
            let hot = match &cv.hover {
                Hit::Handle(id, side) if id == &n.id => Some(*side),
                _ => None,
            };
            paint_plus_handles(
                &painter,
                core_rect(geo.nodes[i]),
                &view,
                origin,
                hot,
                &tokens,
            );
            if let Some(side) = hot {
                paint_ghost(
                    &painter,
                    core_rect(geo.nodes[i]),
                    side,
                    &view,
                    origin,
                    accent,
                );
            }
        }
    }

    // card edge labels sit above the nodes, as in the export
    if cards {
        for (i, e) in diagram.edges.iter().enumerate() {
            let Some(path) = &geo.edges[i] else { continue };
            if path.points.len() < 2 || e.label.trim().is_empty() {
                continue;
            }
            let editing_edge = matches!(&cv.editing, Some(Editing { target: EditTarget::Edge(id), .. }) if id == &e.id);
            if !editing_edge {
                let at = view.to_screen(origin, pt(path.label_at));
                crate::cards::paint_label_chip(&painter, &e.label, at, &brand, zoom, dark);
            }
        }
    }

    // click-click connect: line from the picked node to the pointer
    if let (Some(from), Some(p)) = (&cv.connect_from, ptr_screen)
        && let Some(i) = diagram.nodes.iter().position(|n| &n.id == from)
    {
        let start = view.rect_to_screen(origin, geo.nodes[i]).center();
        painter.add(EShape::dashed_line(
            &[start, p],
            Stroke::new(2.0, accent),
            6.0,
            4.0,
        ));
        painter.rect_stroke(
            view.rect_to_screen(origin, geo.nodes[i]).expand(5.0),
            10,
            Stroke::new(2.0, accent),
            egui::StrokeKind::Outside,
        );
    }

    // gesture overlays
    match &cv.drag {
        Drag::Connect { from, side, .. } => {
            if let (Some(i), Some(p)) =
                (diagram.nodes.iter().position(|n| &n.id == from), ptr_screen)
            {
                let start = match side {
                    Some(s) => {
                        let a = quick::side_anchor(core_rect(geo.nodes[i]), *s);
                        view.to_screen(origin, pos2(a.x, a.y))
                    }
                    None => view.rect_to_screen(origin, geo.nodes[i]).center(),
                };
                arrow_line(&painter, start, p, Stroke::new(1.5, accent));
                let target = ptr_world.map(|w| hit_test(diagram, &geo, w, zoom, &[]));
                if let Some(Hit::Node(id)) = target
                    && &id != from
                    && let Some(j) = diagram.nodes.iter().position(|n| n.id == id)
                {
                    let r = view.rect_to_screen(origin, geo.nodes[j]).expand(5.0);
                    painter.rect_stroke(r, 10, Stroke::new(2.0, accent), egui::StrokeKind::Outside);
                }
            }
        }
        Drag::Create {
            start,
            cancelled: false,
            ..
        } => {
            if let Some(w) = ptr_world {
                let b = quick::drag_box(
                    Point::new(start.x, start.y),
                    Point::new(w.x, w.y),
                    mods.shift,
                );
                let r = view
                    .rect_to_screen(origin, Rect::from_min_size(pos2(b.x, b.y), vec2(b.w, b.h)));
                painter.rect(
                    r,
                    4,
                    accent.gamma_multiply(0.05),
                    Stroke::new(1.0, accent.gamma_multiply(0.45)),
                    egui::StrokeKind::Inside,
                );
            }
        }
        Drag::Marquee { start, .. } => {
            if let Some(p) = ptr_screen {
                let m = Rect::from_two_pos(view.to_screen(origin, *start), p);
                painter.rect(
                    m,
                    2,
                    accent.gamma_multiply(0.12),
                    Stroke::new(1.0, accent),
                    egui::StrokeKind::Inside,
                );
            }
        }
        _ => {}
    }

    if diagram.nodes.is_empty() && cv.editing.is_none() {
        empty_state(ui, rect, left_inset, &tokens, cmds);
    }

    edit_overlay(ui, cv, diagram, &geo, &view, origin, &brand);
    cv.task_highlight = None;
}

/// Make a step from `source`'s handle on `side` (centred on `drop` when a drag was released
/// there), connect it, select it, return to Select and open its label editor.
fn quick_create(
    d: &mut Diagram,
    sel: &mut ops::Selection,
    cv: &mut CanvasState,
    source: &str,
    side: Side,
    drop: Option<Pos2>,
) {
    let Some(src) = d.node(source).cloned() else {
        return;
    };
    let id = d.fresh_id("n");
    let n = quick::plan(
        d,
        &src,
        side,
        id.clone(),
        drop.map(|p| Point::new(p.x, p.y)),
    );
    d.nodes.push(n);
    ops::connect(d, source, &id);
    sel.only_node(&id);
    cv.set_tool(Tool::Select);
    cv.start_edit(&id, d);
    cv.uncover = Some(id);
}

/// A default-size blank step centred on `centre`, or filling `bx`; selected, editor open.
fn place_step(
    d: &mut Diagram,
    sel: &mut ops::Selection,
    cv: &mut CanvasState,
    centre: Pos2,
    bx: Option<rovelo_core::model::Rect>,
) {
    let id = d.fresh_id("n");
    let mut n = rovelo_core::model::Node::new(id.clone(), "");
    n.shape = rovelo_core::model::Shape::Rounded;
    match bx {
        Some(b) => {
            n.size = Some(rovelo_core::model::Size { w: b.w, h: b.h });
            n.pos = Some(Point::new(b.x, b.y));
        }
        None => {
            let s = quick::default_size(d);
            n.pos = Some(Point::new(
                snap(centre.x - s.w / 2.0, GRID),
                snap(centre.y - s.h / 2.0, GRID),
            ));
        }
    }
    d.nodes.push(n);
    sel.only_node(&id);
    cv.set_tool(Tool::Select);
    cv.start_edit(&id, d);
    cv.uncover = Some(id);
}

/// Four 16px plus handles 22px outside the sides; the hot one fills with the accent.
fn paint_plus_handles(
    painter: &egui::Painter,
    r: rovelo_core::model::Rect,
    view: &View,
    origin: Pos2,
    hot: Option<Side>,
    t: &Tokens,
) {
    let radius = quick::HANDLE_PX / 2.0;
    for (side, p) in quick::handle_points(r, view.zoom) {
        let c = view.to_screen(origin, pos2(p.x, p.y));
        let active = hot == Some(side);
        let (fill, ink) = if active {
            (t.accent, t.surface)
        } else {
            (t.surface, t.accent)
        };
        painter.circle(c, radius, fill, Stroke::new(1.25, t.accent));
        let arm = radius / 2.0;
        painter.line_segment(
            [c - vec2(arm, 0.0), c + vec2(arm, 0.0)],
            Stroke::new(1.25, ink),
        );
        painter.line_segment(
            [c - vec2(0.0, arm), c + vec2(0.0, arm)],
            Stroke::new(1.25, ink),
        );
    }
}

/// What a click on the hot handle makes: a same-size box one node size away, with the arrow into it.
fn paint_ghost(
    painter: &egui::Painter,
    r: rovelo_core::model::Rect,
    side: Side,
    view: &View,
    origin: Pos2,
    accent: Color32,
) {
    let g = quick::ghost(r, side);
    let gr = view.rect_to_screen(origin, Rect::from_min_size(pos2(g.x, g.y), vec2(g.w, g.h)));
    painter.rect(
        gr,
        4,
        accent.gamma_multiply(0.05),
        Stroke::new(1.0, accent.gamma_multiply(0.45)),
        egui::StrokeKind::Inside,
    );
    let a = quick::side_anchor(r, side);
    let b = quick::side_anchor(g, side.opposite());
    arrow_line(
        painter,
        view.to_screen(origin, pos2(a.x, a.y)),
        view.to_screen(origin, pos2(b.x, b.y)),
        Stroke::new(1.5, accent.gamma_multiply(0.45)),
    );
}

/// A straight line with an 8px arrowhead at `to`.
fn arrow_line(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    painter.line_segment([from, to], stroke);
    let d = to - from;
    if d.length() < 1.0 {
        return;
    }
    let dir = d.normalized();
    let back = to - dir * 8.0;
    let perp = vec2(-dir.y, dir.x) * 4.0;
    painter.add(EShape::convex_polygon(
        vec![to, back + perp, back - perp],
        stroke.color,
        Stroke::NONE,
    ));
}

fn paint_grid(painter: &egui::Painter, rect: Rect, view: &View, color: Color32) {
    // Double the world spacing until the dots are at least 10px apart.
    let mut world_step = 24.0;
    while world_step * view.zoom < 10.0 && world_step < 1.0e6 {
        world_step *= 2.0;
    }
    let step = world_step * view.zoom;
    let r = (1.0 * view.zoom.sqrt()).clamp(0.9, 1.8);
    let start = view.to_world(rect.min, rect.min);
    let x0 = (start.x / world_step).floor() * world_step;
    let y0 = (start.y / world_step).floor() * world_step;
    let nx = (rect.width() / step).ceil() as i32 + 2;
    let ny = (rect.height() / step).ceil() as i32 + 2;
    if nx * ny > 60_000 {
        return;
    }
    for ix in 0..nx {
        for iy in 0..ny {
            let w = pos2(x0 + ix as f32 * world_step, y0 + iy as f32 * world_step);
            painter.circle_filled(view.to_screen(rect.min, w), r, color);
        }
    }
}

fn dir_or(a: Pos2, b: Pos2, fallback: Vec2) -> Vec2 {
    let d = b - a;
    if d.length() > 1e-3 {
        d.normalized()
    } else {
        fallback
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_edge(
    painter: &egui::Painter,
    e: &rovelo_core::model::Edge,
    path: &EdgePath,
    brand: &BrandKit,
    view: &View,
    origin: Pos2,
    selected: bool,
    accent: Color32,
) {
    let col = rgb(brand.palette.get(e.role.unwrap_or(Role::Neutral)));
    let z = view.zoom;
    let sw_world = brand.stroke_width.max(0.5);
    let sw = (sw_world * z).max(1.0);
    let pts: Vec<Pos2> = path
        .points
        .iter()
        .map(|p| view.to_screen(origin, pt(*p)))
        .collect();
    if pts.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return;
    }
    let head = (8.0 + sw_world * 1.5) * z;
    let n = pts.len();
    let end_dir = dir_or(pts[n - 2], pts[n - 1], vec2(path.end_dir.x, path.end_dir.y));
    let start_dir = dir_or(pts[1], pts[0], vec2(path.start_dir.x, path.start_dir.y));
    let mut line = pts.clone();
    if e.arrow != Arrow::None {
        line[n - 1] -= end_dir * (head * 0.7).min((pts[n - 1] - pts[n - 2]).length());
    }
    if e.arrow == Arrow::Both {
        line[0] -= start_dir * (head * 0.7).min((pts[0] - pts[1]).length());
    }
    if selected {
        painter.add(EShape::line(
            pts.clone(),
            Stroke::new(sw + 6.0, accent.gamma_multiply(0.28)),
        ));
    }
    match e.line {
        LineStyle::Solid => {
            painter.add(EShape::line(line, Stroke::new(sw, col)));
        }
        LineStyle::Dashed => {
            painter.extend(EShape::dashed_line(
                &line,
                Stroke::new(sw, col),
                sw * 5.0,
                sw * 3.5,
            ));
        }
        LineStyle::Dotted => dots(painter, &line, sw * 3.2 + 0.5, sw * 0.5, col),
    }
    let arrow = |tip: Pos2, dir: Vec2| {
        let perp = vec2(-dir.y, dir.x);
        let base = tip - dir * head;
        painter.add(EShape::convex_polygon(
            vec![tip, base + perp * head * 0.5, base - perp * head * 0.5],
            col,
            Stroke::NONE,
        ));
    };
    if e.arrow != Arrow::None {
        arrow(pts[n - 1], end_dir);
    }
    if e.arrow == Arrow::Both {
        arrow(pts[0], start_dir);
    }
    if !e.label.trim().is_empty() {
        let px = font_px(brand, z, 0.86);
        if px >= 5.0 {
            let galley =
                painter.layout_no_wrap(e.label.clone(), FontId::proportional(px), rgb(brand.text));
            let c = view.to_screen(origin, pt(path.label_at));
            let pill = pill_size(&e.label, brand.font_size).unwrap_or(vec2(40.0, 20.0)) * z;
            let r = Rect::from_center_size(
                c,
                vec2(
                    pill.x.max(galley.size().x + 8.0 * z),
                    pill.y.max(galley.size().y + 4.0 * z),
                ),
            );
            painter.rect(
                r,
                (r.height() / 2.0).min(10.0 * z),
                rgb(brand.background),
                Stroke::new(
                    1.0,
                    rgb(brand
                        .palette
                        .get(e.role.unwrap_or(Role::Neutral))
                        .mix(brand.background, 0.45)),
                ),
                egui::StrokeKind::Inside,
            );
            painter.galley(c - galley.size() / 2.0, galley, rgb(brand.text));
        }
    }
}

/// Round dots every `step` px along a polyline.
fn dots(painter: &egui::Painter, pts: &[Pos2], step: f32, radius: f32, color: Color32) {
    let mut carry = 0.0;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = (b - a).length();
        if len < 1e-3 {
            continue;
        }
        let dir = (b - a) / len;
        let mut t = carry;
        while t <= len {
            painter.circle_filled(a + dir * t, radius.max(0.8), color);
            t += step;
        }
        carry = t - len;
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_node(
    painter: &egui::Painter,
    d: &Diagram,
    dark: bool,
    n: &rovelo_core::model::Node,
    wr: Rect,
    sr: Rect,
    brand: &BrandKit,
    zoom: f32,
    selected: bool,
    highlight: bool,
    badge: Option<(usize, usize)>,
    editing: bool,
    accent: Color32,
) {
    // Shape style: the card's neutral surface and 1px border; the role shows in the glyph.
    let ink = rovelo_core::card::ink(brand, dark);
    let colors = rovelo_core::brand::NodeColors {
        fill: ink.surface,
        stroke: ink.border,
        text: ink.text,
    };
    let sw = 1.0;
    let pts = shapes::outline(n.shape, sr, brand.corner_radius, zoom);
    if brand.shadow {
        for (grow, alpha) in [(3.0, 7u8), (2.0, 9), (1.0, 12)] {
            let r = sr
                .translate(vec2(0.0, 2.0 * zoom))
                .expand(grow * zoom * 0.6);
            shapes::fill(
                painter,
                &shapes::outline(n.shape, r, brand.corner_radius + grow * 0.6, zoom),
                Color32::from_rgba_unmultiplied(24, 24, 27, alpha),
            );
        }
    }
    shapes::fill(painter, &pts, rgb(colors.fill));
    shapes::stroke_closed(painter, &pts, Stroke::new(sw, rgb(colors.stroke)));
    for d in shapes::details(n.shape, sr, zoom) {
        match d {
            shapes::Detail::Line(l) => {
                painter.add(EShape::line(l, Stroke::new(sw, rgb(colors.stroke))));
            }
            shapes::Detail::Bar(b) => {
                painter.rect_filled(b, 2.0 * zoom, rgb(colors.stroke).gamma_multiply(0.45));
            }
            shapes::Detail::Fold(f) => {
                shapes::fill(painter, &f, rgb(colors.stroke).gamma_multiply(0.25));
                painter.add(EShape::line(f, Stroke::new(sw, rgb(colors.stroke))));
            }
        }
    }
    if selected || highlight {
        let mut ring = shapes::outline(
            n.shape,
            sr.expand(5.0),
            brand.corner_radius + 5.0 / zoom,
            zoom,
        );
        if highlight && !selected {
            if let Some(f) = ring.first().copied() {
                ring.push(f);
            }
            painter.extend(EShape::dashed_line(
                &ring,
                Stroke::new(2.0, accent),
                5.0,
                4.0,
            ));
        } else {
            shapes::stroke_closed(painter, &ring, Stroke::new(2.0, accent));
        }
    }

    let px = font_px(brand, zoom, 1.0);
    if px >= 4.0 && !editing && !n.label.trim().is_empty() {
        let ink = rgb(colors.text);
        let cy = sr.center().y + shapes::text_offset(n.shape, wr.height()) * zoom;
        let galley = painter.layout_no_wrap(n.label.clone(), FontId::proportional(px), ink);
        let icon = rovelo_core::card::icon_word(d, n)
            .as_deref()
            .and_then(chrome_icons::Icon::for_node_word);
        let glyph = rgb(rovelo_core::card::vibrant(brand, n.role, dark));
        let isz = (brand.font_size * 1.25).round() * zoom;
        let total = isz + 8.0 * zoom + galley.size().x;
        let ix = sr.center().x - total / 2.0;
        match icon {
            Some(name) => chrome_icons::paint(painter, name, pos2(ix + isz / 2.0, cy), isz, glyph),
            None => {
                painter.circle_filled(pos2(ix + isz / 2.0, cy), 3.0 * zoom, glyph);
            }
        }
        let text_x = ix + isz + 8.0 * zoom + galley.size().x / 2.0;
        painter.galley(pos2(text_x, cy) - galley.size() / 2.0, galley, ink);
    }

    if let Some((done, total)) = badge {
        let bs = 10.5 * zoom;
        if bs >= 4.5 {
            let (fill, ink) = if done == total {
                (
                    rovelo_core::brand::Rgb(accent.r(), accent.g(), accent.b()),
                    rovelo_core::brand::Rgb(255, 255, 255),
                )
            } else {
                (brand.text, brand.background)
            };
            let txt = format!("{done}/{total}");
            let (w, h) = (
                (txt.chars().count() as f32 * 0.6 * 10.5 + 14.0) * zoom,
                18.0 * zoom,
            );
            let r = Rect::from_min_size(
                pos2(sr.right() + 6.0 * zoom - w, sr.top() - h / 2.0),
                vec2(w, h),
            );
            painter.rect(
                r,
                9.0 * zoom,
                rgb(fill),
                Stroke::new((2.0 * zoom).max(1.0), rgb(brand.background)),
                egui::StrokeKind::Inside,
            );
            let galley = painter.layout_no_wrap(txt, FontId::proportional(bs), rgb(ink));
            painter.galley(r.center() - galley.size() / 2.0, galley, rgb(ink));
        }
    }
}

/// Small drawn preview of a template's shape, inside `area`.
fn template_preview(painter: &egui::Painter, area: Rect, tpl: Template, t: &Tokens) {
    let line = Stroke::new(1.5, t.border_strong);
    let node = |r: Rect, accent: bool| {
        let s = Stroke::new(1.5, if accent { t.accent } else { t.muted });
        painter.rect(
            r,
            4,
            if accent { t.accent_soft } else { t.sunk },
            s,
            egui::StrokeKind::Inside,
        );
    };
    let c = area.center();
    match tpl {
        Template::UserFlow => {
            let (w, h) = (22.0, 36.0);
            let xs = [-0.34, 0.0, 0.34];
            let rects: Vec<Rect> = xs
                .iter()
                .map(|x| Rect::from_center_size(c + vec2(area.width() * x, 0.0), vec2(w, h)))
                .collect();
            for pair in rects.windows(2) {
                painter.line_segment(
                    [
                        pair[0].right_center() + vec2(2.0, 0.0),
                        pair[1].left_center() - vec2(2.0, 0.0),
                    ],
                    line,
                );
            }
            for (i, r) in rects.iter().enumerate() {
                node(*r, i == 0);
                painter.line_segment(
                    [
                        r.center_top() + vec2(-4.0, 5.0),
                        r.center_top() + vec2(4.0, 5.0),
                    ],
                    Stroke::new(1.5, t.border_strong),
                );
            }
        }
        Template::TaskFlow => {
            let pill =
                Rect::from_center_size(c + vec2(-area.width() * 0.34, -6.0), vec2(30.0, 16.0));
            let dia = c + vec2(0.0, -6.0);
            let rect =
                Rect::from_center_size(c + vec2(area.width() * 0.34, -6.0), vec2(30.0, 16.0));
            let low = Rect::from_center_size(c + vec2(0.0, 18.0), vec2(26.0, 12.0));
            painter.line_segment([pill.right_center(), dia + vec2(-12.0, 0.0)], line);
            painter.line_segment([dia + vec2(12.0, 0.0), rect.left_center()], line);
            painter.line_segment([dia + vec2(0.0, 12.0), low.center_top()], line);
            painter.rect(
                pill,
                8,
                t.accent_soft,
                Stroke::new(1.5, t.accent),
                egui::StrokeKind::Inside,
            );
            painter.add(EShape::convex_polygon(
                vec![
                    dia + vec2(0.0, -12.0),
                    dia + vec2(12.0, 0.0),
                    dia + vec2(0.0, 12.0),
                    dia + vec2(-12.0, 0.0),
                ],
                t.sunk,
                Stroke::new(1.5, t.muted),
            ));
            node(rect, false);
            node(low, false);
        }
        Template::Blank => {
            let r = Rect::from_center_size(c, vec2(56.0, 38.0));
            let mut pts = shapes::rounded_rect(r, 6.0);
            if let Some(f) = pts.first().copied() {
                pts.push(f);
            }
            painter.extend(EShape::dashed_line(
                &pts,
                Stroke::new(1.5, t.border_strong),
                4.0,
                3.0,
            ));
            painter.line_segment(
                [c + vec2(-5.0, 0.0), c + vec2(5.0, 0.0)],
                Stroke::new(1.5, t.muted),
            );
            painter.line_segment(
                [c + vec2(0.0, -5.0), c + vec2(0.0, 5.0)],
                Stroke::new(1.5, t.muted),
            );
        }
    }
}

/// The centred "Start a flow" block: no frame, no backdrop, three template cards.
fn empty_state(ui: &mut Ui, rect: Rect, left_inset: f32, t: &Tokens, cmds: &mut Vec<Cmd>) {
    use crate::theme::m;
    let region = Rect::from_min_max(pos2(rect.left() + left_inset, rect.top()), rect.max);
    let gap = m::SPACE_16;
    let total_w = m::CARD_W * 3.0 + gap * 2.0;
    if region.width() < total_w + 2.0 * m::SPACE_24 {
        return;
    }
    let painter = ui.painter().clone();
    let title = painter.layout_no_wrap(
        "Start a flow".into(),
        FontId::new(m::TEXT_LG, theme::semibold_family()),
        t.text,
    );
    let sub = painter.layout_no_wrap(
        "Pick a starting point. Brand, layout and to-dos can all change later.".into(),
        FontId::proportional(m::TEXT_SM),
        t.muted,
    );
    let hint = painter.layout_no_wrap(
        "or double-click the canvas to add a step".into(),
        FontId::new(m::TEXT_XS, theme::medium_family()),
        t.muted,
    );
    let block_h = title.size().y
        + m::SPACE_8
        + sub.size().y
        + m::SPACE_24
        + m::CARD_H
        + m::SPACE_16
        + hint.size().y;
    let mid = region.center().x.round();
    let mut y = (region.center().y - block_h / 2.0).round();
    painter.galley(pos2(mid - title.size().x / 2.0, y), title.clone(), t.text);
    y += title.size().y + m::SPACE_8;
    painter.galley(pos2(mid - sub.size().x / 2.0, y), sub.clone(), t.muted);
    y += sub.size().y + m::SPACE_24;
    let mut x = mid - total_w / 2.0;
    for tpl in Template::ALL {
        let r = Rect::from_min_size(pos2(x, y), vec2(m::CARD_W, m::CARD_H));
        let id = ui.scope_id().with(("template_card", tpl.name()));
        let resp = ui.interact(r, id, Sense::click());
        let border = if resp.hovered() {
            t.border_strong
        } else {
            t.border
        };
        painter.add(t.hairline().as_shape(r, m::CARD_RADIUS));
        painter.rect(
            r,
            m::CARD_RADIUS,
            t.surface,
            Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );
        // Preview tile: card padding 8, so its radius is 16 − 8 = 8.
        let preview = Rect::from_min_size(
            r.min + vec2(m::CARD_PAD, m::CARD_PAD),
            vec2(m::CARD_W - 2.0 * m::CARD_PAD, m::SPACE_32 * 3.0),
        );
        painter.rect_filled(preview, m::RADIUS_MD, t.sunk);
        template_preview(&painter, preview, tpl, t);
        let text_x = r.left() + m::SPACE_16;
        let name = painter.layout_no_wrap(
            tpl.name().into(),
            FontId::new(m::TEXT_SM, theme::medium_family()),
            t.text,
        );
        let name_h = name.size().y;
        painter.galley(pos2(text_x, preview.bottom() + m::SPACE_8), name, t.text);
        let blurb = painter.layout_no_wrap(
            tpl.blurb().into(),
            FontId::proportional(m::TEXT_XS),
            t.muted,
        );
        painter.with_clip_rect(r.shrink(m::CARD_PAD)).galley(
            pos2(text_x, preview.bottom() + m::SPACE_8 + name_h),
            blurb,
            t.muted,
        );
        theme::focus_ring(ui, &resp, m::CARD_RADIUS);
        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            cmds.push(Cmd::New(tpl));
        }
        x += m::CARD_W + gap;
    }
    y += m::CARD_H + m::SPACE_16;
    painter.galley(pos2(mid - hint.size().x / 2.0, y), hint, t.muted);
}

fn edit_overlay(
    ui: &mut Ui,
    cv: &mut CanvasState,
    d: &mut Diagram,
    geo: &Geo,
    view: &View,
    origin: Pos2,
    brand: &BrandKit,
) {
    let Some(ed) = cv.editing.clone() else { return };
    let (screen_rect, text): (Rect, Option<&mut String>) = match &ed.target {
        EditTarget::Node(id) => {
            let Some(i) = d.nodes.iter().position(|n| &n.id == id) else {
                cv.editing = None;
                return;
            };
            let mut sr = view.rect_to_screen(origin, geo.nodes[i]);
            if rovelo_core::card::uses_cards(brand) {
                // the title area to the right of the icon tile
                use rovelo_core::card;
                sr.min.x +=
                    (card::PAD + card::TILE + card::TILE_GAP - theme::m::SPACE_8) * view.zoom;
                sr.max.x -= (card::PAD - theme::m::SPACE_8).max(0.0) * view.zoom;
            }
            (sr, d.nodes.get_mut(i).map(|n| &mut n.label))
        }
        EditTarget::Edge(id) => {
            let Some(i) = d.edges.iter().position(|e| &e.id == id) else {
                cv.editing = None;
                return;
            };
            let at = geo.edges[i]
                .as_ref()
                .map(|p| view.to_screen(origin, pt(p.label_at)))
                .unwrap_or(origin);
            (
                Rect::from_center_size(at, vec2(140.0, 26.0)),
                d.edges.get_mut(i).map(|e| &mut e.label),
            )
        }
    };
    let Some(text) = text else { return };
    let px = font_px(brand, view.zoom, 1.0)
        .clamp(theme::m::TEXT_XS, theme::m::SPACE_32 + theme::m::SPACE_8);
    let left = rovelo_core::card::uses_cards(brand) && matches!(ed.target, EditTarget::Node(_));
    let width = screen_rect.width().max(120.0);
    let mut r = Rect::from_center_size(screen_rect.center(), vec2(width, (px + 14.0).max(30.0)));
    if left {
        r = r.translate(vec2(screen_rect.left() - r.left(), 0.0));
    }
    let tokens = Tokens::of(ui);
    let accent = tokens.accent;
    let lines = text.lines().count().max(1) as f32;
    let r = Rect::from_center_size(
        r.center(),
        vec2(r.width(), r.height() + (lines - 1.0) * px * 1.3),
    );
    // Enter, Tab or Esc commit and close; Shift+Enter is a new line.
    let enter = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
    let mut finished = enter;
    ui.scope_builder(UiBuilder::new().max_rect(r), |ui| {
        egui::Frame::new()
            .fill(tokens.surface)
            .stroke(Stroke::new(theme::m::FOCUS_W, accent))
            .corner_radius(theme::m::r(theme::m::RADIUS_MD))
            .inner_margin(egui::Margin::symmetric(
                theme::m::px(theme::m::SPACE_8),
                theme::m::px(theme::m::SPACE_4),
            ))
            .show(ui, |ui| {
                let mut out = TextEdit::multiline(text)
                    .font(FontId::proportional(px))
                    .frame(egui::Frame::NONE)
                    .align(if left {
                        egui::Align2::LEFT_CENTER
                    } else {
                        egui::Align2::CENTER_CENTER
                    })
                    .desired_rows(1)
                    .desired_width(ui.available_width())
                    .show(ui);
                if ed.fresh {
                    out.response.request_focus();
                    let len = text.chars().count();
                    out.state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(0),
                            egui::text::CCursor::new(len),
                        )));
                    out.state.store(ui.ctx(), out.response.id);
                } else if out.response.lost_focus() || !out.response.has_focus() {
                    finished = true;
                }
            });
    });
    if let Some(e) = &mut cv.editing {
        e.fresh = false;
    }
    if finished {
        cv.editing = None;
    }
}
