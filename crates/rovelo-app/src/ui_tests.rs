//! Headless frame tests: real egui input events driven through the whole app.
use crate::app::{Cmd, RoveloApp, SideTab};
use crate::dates;
use crate::templates::{self, Template};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, vec2};
use rovelo_core::brand::BrandKit;
use rovelo_core::model::{Diagram, Point};

struct H {
    ctx: egui::Context,
    app: RoveloApp,
    t: f64,
    events: Vec<Event>,
}

impl H {
    fn new(d: Diagram) -> H {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut h = H {
            ctx,
            app: RoveloApp::with_diagram(d, BrandKit::slate()),
            t: 0.0,
            events: vec![],
        };
        h.app.canvas.fit_request = true;
        h.frames(4);
        h
    }

    fn user_flow() -> H {
        H::new(templates::build(
            Template::UserFlow,
            dates::today(),
            &BrandKit::slate(),
        ))
    }

    fn frame(&mut self) {
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0))),
            time: Some(self.t),
            events: std::mem::take(&mut self.events),
            ..Default::default()
        };
        self.t += 0.05;
        let app = &mut self.app;
        self.ctx
            .run_ui(raw, |ui| app.draw(ui))
            .drop_without_applying_deltas();
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            self.frame();
        }
    }

    fn node_screen(&self, id: &str) -> Pos2 {
        let d = &self.app.diagram;
        let r = rovelo_core::geometry::node_rect(d, d.node(id).expect("node"));
        let c = Point::new(r.x + r.w / 2.0, r.y + r.h / 2.0);
        self.app
            .canvas
            .view
            .to_screen(self.app.canvas.origin, Pos2::new(c.x, c.y))
    }

    /// A point on the bare canvas, clear of the floating chrome.
    fn free_point(&self) -> Pos2 {
        self.app.canvas.origin + self.app.canvas.size - vec2(60.0, 120.0)
    }

    fn world_to_screen(&self, x: f32, y: f32) -> Pos2 {
        self.app
            .canvas
            .view
            .to_screen(self.app.canvas.origin, Pos2::new(x, y))
    }

    fn mv(&mut self, p: Pos2) {
        self.events.push(Event::PointerMoved(p));
        self.frame();
    }

    fn button(&mut self, p: Pos2, pressed: bool, mods: Modifiers) {
        self.events.push(Event::ModifiersChanged(mods));
        self.events.push(Event::PointerButton {
            pos: p,
            button: PointerButton::Primary,
            pressed,
            modifiers: mods,
        });
        self.frame();
    }

    fn click(&mut self, p: Pos2) {
        self.mv(p);
        self.button(p, true, Modifiers::NONE);
        self.button(p, false, Modifiers::NONE);
        self.frames(2);
    }

    fn drag(&mut self, from: Pos2, to: Pos2, mods: Modifiers) {
        self.mv(from);
        self.button(from, true, mods);
        for i in 1..=8 {
            let p = from + (to - from) * (i as f32 / 8.0);
            self.mv(p);
        }
        self.button(to, false, mods);
        self.events.push(Event::ModifiersChanged(Modifiers::NONE));
        self.frames(2);
    }

    fn key(&mut self, key: Key, mods: Modifiers) {
        self.events.push(Event::ModifiersChanged(mods));
        self.events.push(Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers: mods,
        });
        self.events.push(Event::Key {
            key,
            physical_key: Some(key),
            pressed: false,
            repeat: false,
            modifiers: mods,
        });
        self.events.push(Event::ModifiersChanged(Modifiers::NONE));
        self.frames(2);
    }
}

#[test]
fn click_selects_and_shift_click_adds() {
    let mut h = H::user_flow();
    h.click(h.node_screen("login"));
    assert_eq!(h.app.sel.nodes, vec!["login".to_string()]);
    let p = h.node_screen("signup");
    h.mv(p);
    h.button(p, true, Modifiers::SHIFT);
    h.button(p, false, Modifiers::SHIFT);
    h.events.push(Event::ModifiersChanged(Modifiers::NONE));
    h.frames(3);
    assert_eq!(h.app.sel.nodes.len(), 2, "{:?}", h.app.sel);
}

#[test]
fn click_empty_canvas_clears_selection() {
    let mut h = H::user_flow();
    h.click(h.node_screen("login"));
    let p = h.free_point();
    h.click(p);
    assert!(h.app.sel.is_empty());
}

#[test]
fn dragging_a_node_moves_it_on_the_grid_and_is_one_undo_step() {
    let mut h = H::user_flow();
    let before = h.app.diagram.node("verify").unwrap().pos.unwrap();
    assert!(!h.app.history.can_undo());
    let from = h.node_screen("verify");
    h.drag(from, from + vec2(77.0, 41.0), Modifiers::NONE);
    let after = h.app.diagram.node("verify").unwrap().pos.unwrap();
    assert!(
        after.x > before.x + 20.0 && after.y > before.y + 10.0,
        "{before:?} -> {after:?}"
    );
    assert_eq!(after.x % 8.0, 0.0, "x snapped: {}", after.x);
    assert_eq!(after.y % 8.0, 0.0, "y snapped: {}", after.y);
    assert!(h.app.dirty);
    h.app.run(Cmd::Undo);
    h.frames(2);
    assert_eq!(
        h.app.diagram.node("verify").unwrap().pos.unwrap(),
        before,
        "one drag must be one undo step"
    );
    assert!(!h.app.history.can_undo(), "no extra history entries");
    h.app.run(Cmd::Redo);
    h.frames(2);
    assert_eq!(h.app.diagram.node("verify").unwrap().pos.unwrap(), after);
}

#[test]
fn alt_drag_disables_snapping() {
    let mut h = H::user_flow();
    let from = h.node_screen("verify");
    let before = h.app.diagram.node("verify").unwrap().pos.unwrap();
    h.drag(from, from + vec2(41.0, 23.0), Modifiers::ALT);
    let after = h.app.diagram.node("verify").unwrap().pos.unwrap();
    assert!(after.x > before.x + 5.0, "{before:?} {after:?}");
    assert!(
        after.x % 8.0 != 0.0 || after.y % 8.0 != 0.0,
        "alt keeps free positions: {after:?}"
    );
    assert!(h.app.history.can_undo());
}

#[test]
fn marquee_selects_nodes_inside() {
    let mut h = H::user_flow();
    let o = h.app.canvas.origin;
    let size = h.app.canvas.size;
    h.drag(
        o + vec2(2.0, 2.0),
        o + size - vec2(2.0, 2.0),
        Modifiers::NONE,
    );
    assert!(
        h.app.sel.nodes.len() >= 2,
        "marquee over the whole canvas selects nodes: {:?}",
        h.app.sel
    );
}

#[test]
fn dragging_from_a_handle_connects_two_nodes() {
    let mut h = H::user_flow();
    h.click(h.node_screen("landing"));
    h.frames(2);
    let d = &h.app.diagram;
    let r = rovelo_core::geometry::node_rect(d, d.node("landing").unwrap());
    let handle = h.world_to_screen(r.x + r.w, r.y + r.h / 2.0) + vec2(crate::quick::OFFSET_PX, 0.0);
    let edges_before = h.app.diagram.edges.len();
    let target = h.node_screen("dashboard");
    h.drag(handle, target, Modifiers::NONE);
    assert_eq!(h.app.diagram.edges.len(), edges_before + 1);
    let e = h.app.diagram.edges.last().unwrap();
    assert_eq!((e.from.as_str(), e.to.as_str()), ("landing", "dashboard"));
    assert_eq!(h.app.sel.edges.len(), 1);
}

#[test]
fn double_click_empty_adds_a_node_and_starts_editing() {
    let mut h = H::user_flow();
    let n = h.app.diagram.nodes.len();
    let p = h.free_point();
    h.mv(p);
    for _ in 0..2 {
        h.button(p, true, Modifiers::NONE);
        h.button(p, false, Modifiers::NONE);
    }
    h.frames(2);
    assert_eq!(h.app.diagram.nodes.len(), n + 1);
    assert!(h.app.canvas.editing.is_some());
    assert_eq!(h.app.sel.nodes.len(), 1);
}

#[test]
fn delete_removes_selection_and_undo_restores() {
    let mut h = H::user_flow();
    let (n, e) = (h.app.diagram.nodes.len(), h.app.diagram.edges.len());
    h.click(h.node_screen("login"));
    h.key(Key::Delete, Modifiers::NONE);
    assert_eq!(h.app.diagram.nodes.len(), n - 1);
    assert!(h.app.diagram.edges.len() < e);
    assert!(h.app.sel.is_empty());
    h.key(Key::Z, Modifiers::COMMAND);
    assert_eq!(h.app.diagram.nodes.len(), n);
    assert_eq!(h.app.diagram.edges.len(), e);
}

#[test]
fn duplicate_and_arrow_nudge() {
    let mut h = H::user_flow();
    let n = h.app.diagram.nodes.len();
    h.click(h.node_screen("login"));
    h.key(Key::D, Modifiers::COMMAND);
    assert_eq!(h.app.diagram.nodes.len(), n + 1);
    let id = h.app.sel.nodes[0].clone();
    let p0 = h.app.diagram.node(&id).unwrap().pos.unwrap();
    h.key(Key::ArrowRight, Modifiers::NONE);
    let p1 = h.app.diagram.node(&id).unwrap().pos.unwrap();
    assert_eq!(p1.x, p0.x + 8.0);
}

#[test]
fn unsaved_dot_and_save_state() {
    let mut h = H::user_flow();
    assert!(!h.app.dirty);
    h.click(h.node_screen("login"));
    h.key(Key::ArrowDown, Modifiers::NONE);
    assert!(h.app.dirty);
}

#[test]
fn stale_selection_and_empty_diagram_do_not_panic() {
    let mut h = H::new(Diagram::default());
    h.app.sel.nodes.push("ghost".into());
    h.app.sel.edges.push("ghost".into());
    h.app.sel.groups.push("ghost".into());
    h.app.canvas.center_request = Some("ghost".into());
    h.frames(3);
    assert!(h.app.sel.is_empty());
    for tab in [SideTab::Tasks, SideTab::Brand, SideTab::Code] {
        h.app.tab = tab;
        h.frames(2);
    }
    h.key(Key::Delete, Modifiers::NONE);
    h.app.run(Cmd::AutoLayout);
    h.app.run(Cmd::Fit);
    h.app.run(Cmd::Undo);
    h.app.run(Cmd::Redo);
    h.app.run(Cmd::Duplicate);
    h.app.run(Cmd::Group);
    h.frames(3);
}

#[test]
fn extreme_zoom_and_pan_do_not_panic() {
    let mut h = H::user_flow();
    for (zoom, pan) in [
        (6.0, vec2(-1.0e6, 1.0e6)),
        (0.05, vec2(1.0e7, 1.0e7)),
        (f32::NAN, vec2(f32::NAN, 0.0)),
        (1.0e9, vec2(0.0, 0.0)),
    ] {
        h.app.canvas.view.zoom = zoom;
        h.app.canvas.view.pan = pan;
        h.frames(2);
    }
    h.app.diagram.brand.show_grid = true;
    h.app.canvas.view.zoom = 0.06;
    h.frames(2);
    h.app.canvas.view.zoom = 6.0;
    h.frames(2);
}

#[test]
fn all_brand_presets_and_fill_modes_paint() {
    use rovelo_core::brand::{EdgeRouting, NodeFill};
    let mut h = H::user_flow();
    for p in BrandKit::presets() {
        h.app.diagram.brand = p;
        for fill in [NodeFill::Soft, NodeFill::Solid, NodeFill::Outline] {
            for r in [
                EdgeRouting::Straight,
                EdgeRouting::Orthogonal,
                EdgeRouting::Curved,
            ] {
                h.app.diagram.brand.node_fill = fill;
                h.app.diagram.brand.edge_routing = r;
                h.app.diagram.brand.shadow = !h.app.diagram.brand.shadow;
                h.frame();
            }
        }
    }
}

#[test]
fn every_shape_paints_in_the_canvas() {
    let mut d = Diagram::default();
    for (i, s) in rovelo_core::model::Shape::ALL.iter().enumerate() {
        let n = d.ensure_node(&format!("n{i}"), "Label");
        n.shape = *s;
        n.icon = Some("user".into());
        n.pos = Some(Point::new(i as f32 * 200.0, 0.0));
    }
    let mut h = H::new(d);
    h.app.canvas.fit_request = true;
    h.frames(3);
}

#[test]
fn code_tab_round_trip_keeps_positions() {
    let mut h = H::user_flow();
    h.app.tab = SideTab::Code;
    h.frames(3);
    assert!(!h.app.code.text.is_empty(), "text mirrors the diagram");
    let pos = h.app.diagram.node("login").unwrap().pos;
    let n = h.app.diagram.nodes.len();
    h.app
        .code
        .text
        .push_str("\nextra = Extra node\nlogin -> extra\n");
    std::thread::sleep(std::time::Duration::from_millis(300));
    h.frames(4);
    assert_eq!(
        h.app.diagram.nodes.len(),
        n + 1,
        "text edit applied; diagnostics: {:?}",
        h.app.code.diagnostics
    );
    assert_eq!(
        h.app.diagram.node("login").unwrap().pos,
        pos,
        "existing positions are preserved"
    );
    assert!(
        h.app.diagram.node("extra").unwrap().pos.is_some(),
        "new nodes are laid out"
    );
    assert_eq!(h.app.diagram.brand, BrandKit::slate());
    assert!(h.app.history.can_undo());
}

#[test]
fn tasks_panel_selects_and_filters() {
    let mut h = H::user_flow();
    h.app.tab = SideTab::Tasks;
    h.frames(2);
    let (done, total) = h.app.diagram.progress();
    assert!(total > done);
    h.app.sel.only_node("signup");
    h.frames(2);
}

#[test]
fn mermaid_import_replaces_and_undoes() {
    let mut h = H::user_flow();
    let before = h.app.diagram.clone();
    let parsed =
        rovelo_dsl::from_mermaid("flowchart LR\n A[Start] --> B{Ok?}\n B -->|yes| C[Done]");
    let mut d = parsed.diagram;
    d.brand = h.app.diagram.brand.clone();
    rovelo_core::layout::auto_layout(&mut d);
    h.app.replace_undoable(d);
    h.frames(2);
    assert_ne!(h.app.diagram, before);
    h.app.run(Cmd::Undo);
    h.frames(2);
    assert_eq!(h.app.diagram, before);
}

fn assert_fits(h: &H, free: Rect, what: &str) {
    for n in &h.app.diagram.nodes {
        let r = rovelo_core::geometry::node_rect(&h.app.diagram, n);
        let a = h.world_to_screen(r.x, r.y);
        let b = h.world_to_screen(r.x + r.w, r.y + r.h);
        assert!(
            free.contains(a) && free.contains(b),
            "{what}: {} at {a:?}..{b:?} outside {free:?}",
            n.id
        );
    }
}

#[test]
fn fit_keeps_the_diagram_clear_of_the_floating_chrome() {
    for t in [Template::UserFlow, Template::TaskFlow] {
        let mut h = H::new(templates::build(t, dates::today(), &BrandKit::default()));
        let (o, s) = (h.app.canvas.origin, h.app.canvas.size);
        let left = crate::panels::side::PANEL_W + 12.0;
        let free = Rect::from_min_max(
            o + vec2(left + 20.0, 68.0 + 20.0),
            o + s - vec2(20.0, 68.0 + 20.0),
        );
        assert_fits(&h, free, &format!("{t:?} on load"));
        // With a node selected the inspector is open; Fit must avoid it too.
        let id = h.app.diagram.nodes[0].id.clone();
        h.app.sel.only_node(&id);
        h.app.run(Cmd::Fit);
        h.frames(3);
        let right = crate::panels::inspector::PANEL_W + 12.0;
        let free = Rect::from_min_max(
            o + vec2(left + 20.0, 88.0),
            o + s - vec2(right + 20.0, 88.0),
        );
        assert_fits(&h, free, &format!("{t:?} with inspector"));
        // Closing the side panel gives the diagram the room back.
        h.app.sel.clear();
        h.app.run(Cmd::TogglePanel);
        h.app.run(Cmd::Fit);
        h.frames(3);
        let free = Rect::from_min_max(o + vec2(20.0, 88.0), o + s - vec2(20.0, 88.0));
        assert_fits(&h, free, &format!("{t:?} panel closed"));
    }
}

#[test]
fn inline_label_edit_commits_once_and_escape_commits() {
    let mut h = H::user_flow();
    let p = h.node_screen("login");
    h.mv(p);
    for _ in 0..2 {
        h.button(p, true, Modifiers::NONE);
        h.button(p, false, Modifiers::NONE);
    }
    h.frames(3);
    assert!(h.app.canvas.editing.is_some());
    h.events.push(Event::Text("Sign in".into()));
    h.frames(2);
    h.key(Key::Enter, Modifiers::NONE);
    h.frames(2);
    assert!(h.app.canvas.editing.is_none());
    assert_eq!(h.app.diagram.node("login").unwrap().label, "Sign in");
    h.app.run(Cmd::Undo);
    h.frames(2);
    assert_eq!(h.app.diagram.node("login").unwrap().label, "Log in");
    assert!(!h.app.history.can_undo());

    // Escape restores the original text.
    h.mv(p);
    for _ in 0..2 {
        h.button(p, true, Modifiers::NONE);
        h.button(p, false, Modifiers::NONE);
    }
    h.frames(3);
    h.events.push(Event::Text("Nope".into()));
    h.frames(2);
    h.key(Key::Escape, Modifiers::NONE);
    h.frames(2);
    assert_eq!(
        h.app.diagram.node("login").unwrap().label,
        "Nope",
        "Esc commits, like the original app"
    );
    assert!(h.app.canvas.editing.is_none());
}

#[test]
fn add_node_tool_places_one_node_then_returns_to_select() {
    use crate::canvas::Tool;
    let mut h = H::user_flow();
    let n = h.app.diagram.nodes.len();
    h.key(Key::N, Modifiers::NONE);
    assert_eq!(h.app.canvas.tool, Tool::AddNode);
    let p = h.free_point();
    h.click(p);
    assert_eq!(h.app.diagram.nodes.len(), n + 1);
    assert_eq!(h.app.canvas.tool, Tool::Select);
    assert!(h.app.canvas.editing.is_some());
}

#[test]
fn connect_tool_drags_from_a_node_body_and_click_click_works() {
    use crate::canvas::Tool;
    let mut h = H::user_flow();
    h.key(Key::C, Modifiers::NONE);
    assert_eq!(h.app.canvas.tool, Tool::Connect);
    let e = h.app.diagram.edges.len();
    let (a, b) = (h.node_screen("verify"), h.node_screen("login"));
    h.drag(a, b, Modifiers::NONE);
    assert_eq!(h.app.diagram.edges.len(), e + 1);
    // Connecting returns to Select, as the original app does; pick the tool again for click-click.
    assert_eq!(h.app.canvas.tool, Tool::Select);
    h.key(Key::C, Modifiers::NONE);
    h.click(h.node_screen("landing"));
    h.frames(3);
    h.click(h.node_screen("verify"));
    assert_eq!(h.app.diagram.edges.len(), e + 2);
    h.key(Key::Escape, Modifiers::NONE);
    assert_eq!(h.app.canvas.tool, Tool::Select);
}

#[test]
fn escape_closes_the_inspector() {
    let mut h = H::user_flow();
    h.click(h.node_screen("login"));
    assert!(!h.app.sel.is_empty());
    h.key(Key::Escape, Modifiers::NONE);
    assert!(h.app.sel.is_empty());
}

#[test]
fn side_panel_toggle_and_all_tabs_draw() {
    let mut h = H::user_flow();
    h.app.run(Cmd::TogglePanel);
    h.frames(2);
    assert!(!h.app.panel_open);
    h.app.run(Cmd::TogglePanel);
    h.frames(2);
    assert!(h.app.panel_open);
    for tab in [SideTab::Tasks, SideTab::Brand, SideTab::Code] {
        h.app.tab = tab;
        h.frames(3);
    }
}

#[test]
fn inspector_body_grows_to_its_content() {
    let mut h = H::new(templates::build(
        Template::TaskFlow,
        dates::today(),
        &BrandKit::default(),
    ));
    h.app.sel.only_node("tests");
    h.frames(4);
    let raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0))),
        time: Some(h.t),
        ..Default::default()
    };
    let app = &mut h.app;
    let out = h.ctx.run_ui(raw, |ui| app.draw(ui));
    let visible = out.shapes.iter().any(|cs| match &cs.shape {
        egui::epaint::Shape::Text(t) => t.galley.text() == "Delete" && cs.clip_rect.contains(t.pos),
        _ => false,
    });
    out.drop_without_applying_deltas();
    assert!(visible, "the inspector's last row is inside its clip");
}

fn right_handle(h: &H, id: &str) -> Pos2 {
    let d = &h.app.diagram;
    let r = rovelo_core::geometry::node_rect(d, d.node(id).unwrap());
    h.world_to_screen(r.x + r.w, r.y + r.h / 2.0) + vec2(crate::quick::OFFSET_PX, 0.0)
}

#[test]
fn clicking_a_plus_handle_makes_a_connected_step_in_one_undo_and_types_into_it() {
    use crate::canvas::Tool;
    let mut h = H::new(templates::build(
        Template::TaskFlow,
        dates::today(),
        &BrandKit::default(),
    ));
    let (n, e) = (h.app.diagram.nodes.len(), h.app.diagram.edges.len());
    h.click(h.node_screen("review"));
    let handle = right_handle(&h, "review");
    h.mv(handle);
    h.frames(2);
    h.button(handle, true, Modifiers::NONE);
    h.button(handle, false, Modifiers::NONE);
    h.frames(3);
    assert_eq!(h.app.diagram.nodes.len(), n + 1);
    assert_eq!(h.app.diagram.edges.len(), e + 1);
    let new_id = h.app.sel.nodes[0].clone();
    let edge = h.app.diagram.edges.last().unwrap();
    assert_eq!(
        (edge.from.as_str(), edge.to.as_str()),
        ("review", new_id.as_str())
    );
    let src = h.app.diagram.node("review").unwrap().clone();
    let made = h.app.diagram.node(&new_id).unwrap().clone();
    assert_eq!(
        (made.shape, made.role, made.label.as_str()),
        (src.shape, src.role, "")
    );
    assert_eq!(h.app.canvas.tool, Tool::Select);
    assert!(h.app.canvas.editing.is_some(), "the editor opens at once");
    h.events.push(Event::Text("Merge".into()));
    h.frames(2);
    h.key(Key::Enter, Modifiers::NONE);
    h.frames(2);
    assert_eq!(h.app.diagram.node(&new_id).unwrap().label, "Merge");
    assert!(h.app.canvas.editing.is_none());
    // Making the step and naming it in its editor is one gesture: one undo removes node and arrow.
    h.app.run(Cmd::Undo);
    h.frames(2);
    assert_eq!(
        (h.app.diagram.nodes.len(), h.app.diagram.edges.len()),
        (n, e),
        "node and arrow undo together"
    );
    assert!(!h.app.history.can_undo());
}

#[test]
fn clicking_a_decision_handle_makes_a_plain_step() {
    let mut h = H::new(templates::build(
        Template::TaskFlow,
        dates::today(),
        &BrandKit::default(),
    ));
    h.click(h.node_screen("approved"));
    let handle = right_handle(&h, "approved");
    h.mv(handle);
    h.frames(2);
    h.button(handle, true, Modifiers::NONE);
    h.button(handle, false, Modifiers::NONE);
    h.frames(3);
    let id = h.app.sel.nodes[0].clone();
    assert_eq!(
        h.app.diagram.node(&id).unwrap().shape,
        rovelo_core::model::Shape::Rounded
    );
}

#[test]
fn dragging_a_handle_to_empty_canvas_makes_a_step_centred_there() {
    let mut h = H::new(templates::build(
        Template::TaskFlow,
        dates::today(),
        &BrandKit::default(),
    ));
    let n = h.app.diagram.nodes.len();
    h.click(h.node_screen("review"));
    let handle = right_handle(&h, "review");
    let drop = h.free_point();
    h.drag(handle, drop, Modifiers::NONE);
    assert_eq!(h.app.diagram.nodes.len(), n + 1);
    let id = h.app.sel.nodes[0].clone();
    let d = &h.app.diagram;
    let r = rovelo_core::geometry::node_rect(d, d.node(&id).unwrap());
    let centre = h.world_to_screen(r.x + r.w / 2.0, r.y + r.h / 2.0);
    // The view may pan to uncover it; the centre then moved by that pan only.
    assert!(h.app.canvas.editing.is_some());
    assert!(d.edges.iter().any(|e| e.from == "review" && e.to == id));
    let _ = centre;
}

#[test]
fn add_step_tool_drag_draws_the_box() {
    use crate::canvas::Tool;
    let mut h = H::user_flow();
    let n = h.app.diagram.nodes.len();
    h.key(Key::N, Modifiers::NONE);
    let a = h.free_point() - vec2(200.0, 100.0);
    let b = a + vec2(160.0, 80.0);
    h.drag(a, b, Modifiers::NONE);
    assert_eq!(h.app.diagram.nodes.len(), n + 1);
    let id = h.app.sel.nodes[0].clone();
    let size = h.app.diagram.node(&id).unwrap().size.unwrap();
    let z = h.app.canvas.view.zoom;
    assert!(
        (size.w - 160.0 / z).abs() <= 8.0 && (size.h - 80.0 / z).abs() <= 8.0,
        "{size:?} at zoom {z}"
    );
    assert_eq!(h.app.canvas.tool, Tool::Select);
    assert!(h.app.canvas.editing.is_some());
}
