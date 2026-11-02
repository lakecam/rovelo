mod app;
mod canvas;
mod cards;
mod chrome_icons;
mod dates;
mod export;
mod files;
mod ops;
mod panels;
mod quick;
mod shapes;
mod templates;
mod theme;
#[cfg(test)]
mod ui_tests;
mod view;
mod widgets;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([980.0, 600.0])
            .with_title("Rovelo"),
        ..Default::default()
    };
    eframe::run_native(
        "Rovelo",
        options,
        Box::new(|cc| Ok(Box::new(app::RoveloApp::new(cc)))),
    )
}
