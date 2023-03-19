mod geometry;
mod screen_transform;
mod ui;

use eframe::egui;
use ui::State;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(2000.0, 2000.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(State::new())),
    )
}
