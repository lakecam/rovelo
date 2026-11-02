use crate::app::RoveloApp;
use crate::chrome_icons::Icon;
use crate::dates::{self, Date};
use crate::ops;
use crate::theme::{self, Tokens, m};
use egui::{Color32, FontId, Key, RichText, Sense, Stroke, Ui, UiBuilder, pos2, vec2};
use rovelo_core::model::{Id, Priority, Task};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Open,
    Done,
    Overdue,
}

pub fn matches(f: Filter, t: &Task, today: Date) -> bool {
    match f {
        Filter::All => true,
        Filter::Open => !t.done,
        Filter::Done => t.done,
        Filter::Overdue => dates::is_overdue(t.due.as_deref(), t.done, today),
    }
}

/// Where a new to-do attaches.
#[derive(Clone, Debug, Default, PartialEq)]
enum Link {
    /// The selected node if exactly one is selected, else nothing.
    #[default]
    Auto,
    Unlinked,
    Node(Id),
}

#[derive(Default)]
pub struct TasksUi {
    filter: Filter,
    new_title: String,
    link: Link,
    expanded: Option<Id>,
    due_bufs: HashMap<Id, String>,
    tag_bufs: HashMap<Id, String>,
}

pub fn parse_tags(s: &str) -> Vec<String> {
    s.split([',', ' '])
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}\u{2026}",
            s.chars().take(max - 1).collect::<String>().trim_end()
        )
    }
}

/// Indent that lines a row's second line up with its title.
const TITLE_INDENT: f32 = m::CHECKBOX + m::ROW_TITLE_GAP;

pub fn show(app: &mut RoveloApp, ui: &mut Ui) {
    let RoveloApp {
        diagram,
        sel,
        tasks_ui: tu,
        canvas,
        today,
        ..
    } = app;
    let tk = Tokens::of(ui);
    let today = *today;
    let (done, total) = diagram.progress();

    // ---- form part: progress, filter, add --------------------------------
    let auto_node = (sel.nodes.len() == 1).then(|| sel.nodes[0].clone());
    let link_node: Option<Id> = match &tu.link {
        Link::Auto => auto_node.clone(),
        Link::Unlinked => None,
        Link::Node(id) => diagram.node(id).map(|n| n.id.clone()),
    };
    let mut add = false;
    theme::form(ui, |ui| {
        ui.spacing_mut().item_spacing.y = m::SPACE_8;
        ui.horizontal(|ui| {
            ui.label(theme::med(format!("{done} of {total} done"), m::TEXT_SM));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let pct = (done * 100).checked_div(total).unwrap_or(0);
                theme::muted(ui, format!("{pct}%"));
            });
        });
        let (bar, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), m::SPACE_4), Sense::hover());
        ui.painter().rect_filled(bar, m::SPACE_4 / 2.0, tk.sunk);
        if total > 0 && done > 0 {
            let mut fill = bar;
            fill.set_width(bar.width() * done as f32 / total as f32);
            ui.painter().rect_filled(fill, m::SPACE_4 / 2.0, tk.accent);
        }
        ui.add_space(m::SPACE_8);
        let count = |f: Filter| {
            diagram
                .tasks
                .iter()
                .filter(|t| matches(f, t, today))
                .count()
        };
        let labels = [
            format!("All {}", count(Filter::All)),
            format!("Open {}", count(Filter::Open)),
            format!("Done {}", count(Filter::Done)),
            "Overdue".to_string(),
        ];
        theme::segmented(
            ui,
            &mut tu.filter,
            &[
                (Filter::All, labels[0].as_str()),
                (Filter::Open, labels[1].as_str()),
                (Filter::Done, labels[2].as_str()),
                (Filter::Overdue, labels[3].as_str()),
            ],
        );
        ui.add_space(m::SPACE_8);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = m::SPACE_8;
            let add_w = m::SPACE_32 + 2.0 * m::CONTROL_PAD_X;
            let w = (ui.available_width() - add_w - m::SPACE_8).max(m::SPACE_32 * 2.0);
            let r = theme::input_sized(ui, &mut tu.new_title, "Add a to-do\u{2026}", w);
            if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                add = true;
                r.request_focus();
            }
            if theme::button_enabled(ui, !tu.new_title.trim().is_empty(), "Add").clicked() {
                add = true;
            }
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = m::SPACE_8;
            theme::field(ui, "Attach to");
            let mut choice: Option<Link> = None;
            let current = link_node
                .as_ref()
                .and_then(|i| diagram.node(i))
                .map(|n| n.label.clone())
                .unwrap_or_else(|| "No node".into());
            theme::combo("task_link")
                .selected_text(truncate(&current, 22))
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(tu.link == Link::Unlinked, "No node")
                        .clicked()
                    {
                        choice = Some(Link::Unlinked);
                    }
                    for n in &diagram.nodes {
                        let label = if n.label.trim().is_empty() {
                            n.id.clone()
                        } else {
                            n.label.clone()
                        };
                        if ui
                            .selectable_label(link_node.as_deref() == Some(n.id.as_str()), label)
                            .clicked()
                        {
                            choice = Some(Link::Node(n.id.clone()));
                        }
                    }
                });
            if let Some(c) = choice {
                tu.link = c;
            }
        });
    });
    if add && ops::add_task(diagram, &tu.new_title.clone(), link_node.as_deref()).is_some() {
        tu.new_title.clear();
    }

    if diagram.tasks.is_empty() {
        theme::form(ui, |ui| {
            ui.add_space(m::SPACE_16);
            ui.vertical_centered(|ui| {
                ui.label(theme::med("No to-dos yet", m::TEXT_SM));
                theme::muted(ui, "Add one above, or select a node and add it there.");
            });
        });
        return;
    }

    // ---- list part: rows 8 from the panel edge -----------------------------
    let labels: HashMap<Id, String> = diagram
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.label.clone()))
        .collect();
    let nodes: Vec<(Id, String)> = diagram
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.label.clone()))
        .collect();
    let mut delete: Option<Id> = None;
    let mut select: Option<Id> = None;
    let mut shown = 0;
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = m::SPACE_4;
            for t in diagram.tasks.iter_mut() {
                if !matches(tu.filter, t, today) {
                    continue;
                }
                shown += 1;
                let linked_sel = t.node.as_ref().is_some_and(|n| sel.has_node(n));
                let expanded = tu.expanded.as_deref() == Some(t.id.as_str());
                let selected = linked_sel || expanded;
                let overdue = dates::is_overdue(t.due.as_deref(), t.done, today);
                let hover_id = egui::Id::unique(("task_row_hover", &t.id));
                let row = ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
                    let hovered = ui.data(|d| d.get_temp::<bool>(hover_id)).unwrap_or(false);
                    let fill = if selected {
                        theme::selected_fill(&tk)
                    } else if hovered {
                        tk.sunk.gamma_multiply(if tk.dark { 0.6 } else { 1.0 })
                    } else {
                        Color32::TRANSPARENT
                    };
                    egui::Frame::new()
                        .fill(fill)
                        .corner_radius(m::r(m::ROW_RADIUS))
                        .inner_margin(m::px(m::ROW_PAD))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_min_height(m::ROW_MIN_H - 2.0 * m::ROW_PAD);
                            ui.spacing_mut().item_spacing = vec2(m::ROW_TITLE_GAP, m::SPACE_4);
                            ui.horizontal(|ui| {
                                ui.set_min_height(m::ROW_MIN_H - 2.0 * m::ROW_PAD);
                                theme::checkbox(ui, &mut t.done);
                                let ink = if t.done {
                                    tk.muted
                                } else if selected {
                                    theme::selected_ink(&tk)
                                } else {
                                    tk.text
                                };
                                let r = ui.add(
                                    egui::TextEdit::singleline(&mut t.title)
                                        .frame(egui::Frame::NONE)
                                        .margin(egui::Margin::ZERO)
                                        .align(egui::Align2::LEFT_CENTER)
                                        .min_size(vec2(0.0, m::ROW_MIN_H - 2.0 * m::ROW_PAD))
                                        .desired_width(ui.available_width())
                                        .text_color(ink),
                                );
                                if t.done {
                                    let w = ui
                                        .painter()
                                        .layout_no_wrap(
                                            t.title.clone(),
                                            FontId::proportional(m::TEXT_SM),
                                            tk.muted,
                                        )
                                        .size()
                                        .x;
                                    let y = r.rect.center().y;
                                    ui.painter().line_segment(
                                        [
                                            pos2(r.rect.left(), y),
                                            pos2(r.rect.left() + w.min(r.rect.width()), y),
                                        ],
                                        Stroke::new(1.0, tk.muted),
                                    );
                                }
                                if r.gained_focus() {
                                    select = t.node.clone();
                                    tu.expanded = Some(t.id.clone());
                                }
                            });
                            let has_meta = t.due.is_some()
                                || t.node.is_some()
                                || !t.tags.is_empty()
                                || t.priority == Priority::High;
                            if has_meta {
                                ui.horizontal_top(|ui| {
                                    ui.add_space(TITLE_INDENT - m::ROW_TITLE_GAP);
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing =
                                            vec2(m::CHIP_GAP, m::CHIP_GAP);
                                        if let Some(due) = t.due.as_deref() {
                                            let parsed = Date::parse(due);
                                            let text = parsed
                                                .map(|d| d.short(today))
                                                .unwrap_or_else(|| due.to_string());
                                            let due_today = !t.done && parsed == Some(today);
                                            let (txt, col) = if overdue {
                                                (format!("! {text}"), tk.danger)
                                            } else if due_today {
                                                ("Today".to_string(), tk.warning)
                                            } else {
                                                (text, tk.muted)
                                            };
                                            let border = if overdue {
                                                tk.danger
                                            } else if due_today {
                                                tk.warning
                                            } else {
                                                tk.border
                                            };
                                            let r = theme::chip_icon(
                                                ui,
                                                Some(Icon::Calendar),
                                                &txt,
                                                col,
                                                border,
                                                false,
                                            );
                                            if let Some(d) = parsed {
                                                r.on_hover_text(dates::relative(d, today));
                                            }
                                        }
                                        if let Some(l) = t.node.as_ref().and_then(|n| labels.get(n))
                                        {
                                            let r = theme::chip_icon(
                                                ui,
                                                Some(Icon::Link),
                                                &truncate(l, 14),
                                                tk.text,
                                                tk.border,
                                                true,
                                            );
                                            if r.on_hover_text(format!(
                                                "Select and centre \u{201c}{l}\u{201d}"
                                            ))
                                            .clicked()
                                            {
                                                select = t.node.clone();
                                            }
                                        }
                                        if t.priority == Priority::High {
                                            theme::chip(ui, "High", tk.muted, tk.border, false);
                                        }
                                        for tag in &t.tags {
                                            theme::chip(
                                                ui,
                                                &format!("#{tag}"),
                                                tk.muted,
                                                Color32::TRANSPARENT,
                                                false,
                                            );
                                        }
                                    });
                                });
                            }
                            if expanded {
                                ui.add_space(m::SPACE_8);
                                edit_details(ui, t, tu, &nodes, &tk, &mut delete);
                            }
                        });
                });
                let row_hovered = row.response.hovered();
                ui.data_mut(|d| d.insert_temp(hover_id, row_hovered));
                if row_hovered {
                    canvas.task_highlight = t.node.clone();
                }
                if row.response.clicked() {
                    tu.expanded = if expanded { None } else { Some(t.id.clone()) };
                    select = t.node.clone();
                }
            }
            if shown == 0 {
                ui.add_space(m::SPACE_16);
                ui.vertical_centered(|ui| theme::muted(ui, "Nothing matches this filter."));
            }
        });
    if let Some(id) = select
        && diagram.node(&id).is_some()
    {
        sel.only_node(&id);
        canvas.center_request = Some(id);
    }
    if let Some(id) = delete {
        diagram.tasks.retain(|t| t.id != id);
        tu.expanded = None;
    }
}

/// Buffered single-line input; the buffer follows the model while the field is unfocused.
fn buffered_input(
    ui: &mut Ui,
    id: egui::Id,
    buf: &mut String,
    model: String,
    hint: &str,
    ink: Color32,
) -> egui::Response {
    let t = Tokens::of(ui);
    if !ui.memory(|mm| mm.has_focus(id)) {
        *buf = model;
    }
    let r = ui.add(
        theme::input_edit(buf, hint, &t)
            .id(id)
            .desired_width(ui.available_width())
            .text_color(ink),
    );
    theme::focus_input_ring(ui, &r);
    r
}

fn edit_details(
    ui: &mut Ui,
    t: &mut Task,
    tu: &mut TasksUi,
    nodes: &[(Id, String)],
    tk: &Tokens,
    delete: &mut Option<Id>,
) {
    ui.spacing_mut().item_spacing.y = m::LABEL_GAP;
    theme::field(ui, "Due date");
    {
        let id = egui::Id::unique(("due", &t.id));
        let buf = tu.due_bufs.entry(t.id.clone()).or_default();
        let focused = ui.memory(|mm| mm.has_focus(id));
        let probe = if focused {
            buf.clone()
        } else {
            t.due.clone().unwrap_or_default()
        };
        let valid = probe.trim().is_empty() || Date::parse(&probe).is_some();
        let r = buffered_input(
            ui,
            id,
            buf,
            t.due.clone().unwrap_or_default(),
            "YYYY-MM-DD",
            if valid { tk.text } else { tk.danger },
        );
        if r.changed() {
            if buf.trim().is_empty() {
                t.due = None;
            } else if let Some(d) = Date::parse(buf) {
                t.due = Some(d.to_string());
            }
        }
        if !valid {
            ui.label(
                RichText::new("! Use YYYY-MM-DD")
                    .size(m::TEXT_XS)
                    .color(tk.danger),
            );
        }
    }
    theme::field(ui, "Tags");
    {
        let id = egui::Id::unique(("tags", &t.id));
        let buf = tu.tag_bufs.entry(t.id.clone()).or_default();
        let r = buffered_input(ui, id, buf, t.tags.join(", "), "design, copy", tk.text);
        if r.changed() {
            t.tags = parse_tags(buf);
        }
    }
    theme::field(ui, "Priority");
    theme::segmented(
        ui,
        &mut t.priority,
        &[
            (Priority::Low, "Low"),
            (Priority::Normal, "Normal"),
            (Priority::High, "High"),
        ],
    );
    theme::field(ui, "Node");
    ui.horizontal(|ui| {
        let current = t
            .node
            .as_ref()
            .and_then(|n| nodes.iter().find(|(i, _)| i == n))
            .map(|(_, l)| truncate(l, 16))
            .unwrap_or_else(|| "No node".into());
        theme::combo(("tasknode", &t.id))
            .selected_text(current)
            .width(m::SPACE_32 * 4.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(t.node.is_none(), "No node").clicked() {
                    t.node = None;
                }
                for (id, label) in nodes {
                    if ui
                        .selectable_label(t.node.as_deref() == Some(id.as_str()), label)
                        .clicked()
                    {
                        t.node = Some(id.clone());
                    }
                }
            });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if theme::icon_text_button(ui, Icon::Trash, "Delete", true).clicked() {
                *delete = Some(t.id.clone());
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODAY: Date = Date {
        y: 2026,
        m: 11,
        d: 2,
    };

    fn task(done: bool, due: Option<&str>) -> Task {
        let mut t = Task::new("t", "x");
        t.done = done;
        t.due = due.map(str::to_string);
        t
    }

    #[test]
    fn filters() {
        let open = task(false, None);
        let done = task(true, None);
        let late = task(false, Some("2026-10-30"));
        let late_done = task(true, Some("2026-10-30"));
        assert!(matches(Filter::All, &open, TODAY));
        assert!(matches(Filter::Open, &open, TODAY) && !matches(Filter::Open, &done, TODAY));
        assert!(matches(Filter::Done, &done, TODAY) && !matches(Filter::Done, &open, TODAY));
        assert!(matches(Filter::Overdue, &late, TODAY));
        assert!(!matches(Filter::Overdue, &late_done, TODAY));
        assert!(!matches(Filter::Overdue, &open, TODAY));
    }

    #[test]
    fn tag_parsing() {
        assert_eq!(
            parse_tags("design, copy  dev,,"),
            vec!["design", "copy", "dev"]
        );
        assert!(parse_tags("  ").is_empty());
    }

    #[test]
    fn truncation_is_char_safe() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("a very long node label", 10), "a very lo\u{2026}");
        assert_eq!(
            truncate("\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}", 4),
            "\u{00e9}\u{00e9}\u{00e9}\u{2026}"
        );
    }
}
