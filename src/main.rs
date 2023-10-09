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
use std::any::Any;
use std::time::Duration;

//TODO add + in every row with a production or drag and drop to move items to the center panel
//TODO load images on a separate thread so it doesn't lag ui, display buffering circle(egui does it already)?,
// - perhaps make the request on a separate thread in the first place
//TODO temporary workaround for wininit compilation time(15s)?
//TODO add adult checkbox somewhere?
//TODO add exception handling to requests to avoid crashing the app in case something goes wrong

fn main() {
    println!("Running!");
    let config = Config::read_config("config.json");

    let mut options = eframe::NativeOptions::default();
    options.min_window_size = Some(Vec2::new(30f32, 30f32));
    options.drag_and_drop_support = true;
    let app_creator: AppCreator = Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut window = MovieApp::new(cc, config);
        window.setup();
        Box::new(window)
    });
    let _ = eframe::run_native("App", options, app_creator); //blocking call
}
