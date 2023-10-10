mod app;
mod config;
mod credits;
mod production;
mod show_details;
mod themoviedb;

use crate::app::MovieApp;
use crate::config::Config;
use eframe::egui::Vec2;
use eframe::AppCreator;

// TODO: Add + in every row with a production or drag and drop to move items to the center panel
// TODO: Load images on a separate thread so it doesn't lag ui, display buffering circle(egui does it already)?
//       Perhaps make the request on a separate thread in the first place
// TODO: Temporary workaround for winit compilation time (15s)?
// TODO: Add exception handling to requests to avoid crashing the app in case something goes wrong
// TODO: Add scaling to the posters and trim long descriptions, don't artificially stretch the left
//       panel when production entries are added.

fn main() {
    println!("Running!");
    let config = Config::read_config("config.json");

    let mut options = eframe::NativeOptions::default();
    options.min_window_size = Some(Vec2::new(30.0, 30.0));
    options.drag_and_drop_support = true;
    let app_creator: AppCreator = Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut window = MovieApp::new(cc, config);
        window.setup();
        Box::new(window)
    });

    // Blocks the main thread.
    let _ = eframe::run_native("App", options, app_creator);
}
