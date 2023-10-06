mod themoviedb;
mod app;
mod config;

use std::any::Any;
use std::time::Duration;
use eframe::AppCreator;
use eframe::egui::Vec2;
use crate::app::MovieApp;
use crate::config::Config;

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

