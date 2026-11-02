//! The floating 300px left panel with the Tasks / Brand / Code switch.
use crate::app::{RoveloApp, SideTab};
use crate::chrome_icons::Icon;
use crate::panels::{brand, code, tasks};
use crate::theme::{self, Tokens, m};
use egui::{Align2, Area, Context, Id, Order, vec2};

pub const PANEL_W: f32 = 300.0;

pub fn show(app: &mut RoveloApp, ctx: &Context) {
    let fade = ctx.animate_bool_with_time(Id::unique("side_panel_fade"), app.panel_open, m::FADE_S);
    if fade <= 0.0 {
        return;
    }
    let t = Tokens::from_dark(ctx.global_style().visuals.dark_mode);
    // Top at 72; the bottom keeps the zoom pill row (16 + 40 + 16) clear.
    let h = (ctx.content_rect().height() - m::PANEL_TOP - m::PANEL_TOP).max(m::SPACE_32 * 8.0);
    Area::new(Id::unique("side_panel"))
        .order(Order::Middle)
        .fade_in(false)
        .anchor(Align2::LEFT_TOP, vec2(m::EDGE, m::PANEL_TOP))
        .show(ctx, |ui| {
            ui.set_opacity(fade);
            theme::panel_frame(&t).show(ui, |ui| {
                ui.set_width(PANEL_W - 2.0 * m::PANEL_PAD_LIST);
                ui.set_height(h - 2.0 * m::PANEL_PAD_LIST);
                ui.spacing_mut().item_spacing.y = 0.0;
                let (done, total) = app.diagram.progress();
                let tasks_label = if total == 0 {
                    "Tasks".to_string()
                } else {
                    format!("Tasks {done}/{total}")
                };
                theme::form(ui, |ui| {
                    theme::segmented_icons(
                        ui,
                        &mut app.tab,
                        &[
                            (SideTab::Tasks, Some(Icon::Tasks), tasks_label.as_str()),
                            (SideTab::Brand, Some(Icon::Brand), "Brand"),
                            (SideTab::Code, Some(Icon::Code), "Code"),
                        ],
                    );
                });
                match app.tab {
                    SideTab::Tasks => tasks::show(app, ui),
                    SideTab::Brand => brand::show(app, ui),
                    SideTab::Code => code::show(app, ui),
                }
            });
        });
}
