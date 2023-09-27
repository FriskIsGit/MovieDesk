mod themoviedb;
mod app;

use std::any::Any;
use eframe::AppCreator;
use eframe::egui::Vec2;
use crate::app::MovieApp;


#[tokio::main]
async fn main() {
    println!("Running!");
    read_config();
    let mut options = eframe::NativeOptions::default();
    options.min_window_size = Some(Vec2::new(30f32, 30f32));
    options.drag_and_drop_support = true;
    let app_creator: AppCreator = Box::new(|cc| {
        let mut window = MovieApp::new(cc);
        window.setup();
        Box::new(window)
    });
    let _ = eframe::run_native("App", options, app_creator); //blocking call
}

fn read_config(){

}
