mod app;
mod hardware;
mod profile_service;
mod screens;
mod state;

use app::Th80App;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("TH80 Studio")
            .with_inner_size([1180.0, 780.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "TH80 Studio",
        options,
        Box::new(|cc| Ok(Box::new(Th80App::new(cc)))),
    )
}
