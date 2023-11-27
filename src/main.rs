mod app;
mod backend;
mod config;
mod credits;
mod jobs;
mod limiter;
mod movies;
mod production;
mod series;
mod themoviedb;
mod view;

pub const LICENSE: &str = include_str!("../LICENSE.md");

// TODO: Add exception handling to all requests that can fail to avoid crashing the app

fn main() {
    #[cfg(feature = "glfw_backend")]
    backend::glfw_backend::run_app();

    #[cfg(feature = "sdl_backend")]
    backend::sdl_backend::run_app();

    #[cfg(feature = "eframe_backend")]
    backend::eframe_backend::run_app();
}
