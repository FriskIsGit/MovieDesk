mod app;
mod view;
mod backend;
mod config;
mod credits;
mod jobs;
mod production;
mod series_details;
mod themoviedb;
mod limiter;
mod movie_details;

pub const LICENSE: &str = include_str!("../LICENSE.md");

// TODO: Convert no_image.png to .svg
// TODO: Drag and drop to move items to the center panel?
// TODO: Add exception handling to all requests that can fail to avoid crashing the app

fn main() {
    #[cfg(feature = "sdl_backend")]
    backend::sdl_backend::run_app();

    #[cfg(feature = "eframe_backend")]
    backend::eframe_backend::run_app();
}


