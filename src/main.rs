mod app;
mod config;
mod credits;
mod production;
mod series_details;
mod themoviedb;
mod backend;
mod jobs;

// TODO: Add + in every row with a production or drag and drop to move items to the center panel
// TODO: Load images on a separate thread so it doesn't lag ui, display buffering circle(egui does it already)?
//       Perhaps make the request on a separate thread in the first place
// TODO: Temporary workaround for winit compilation time (15s)?
// TODO: Add exception handling to requests to avoid crashing the app in case something goes wrong
// TODO: Add scaling to the posters and trim long descriptions, don't artificially stretch the left
//       panel when production entries are added.
// TODO: Add a title filtering field 'Your movies' - visible only if there are at least 3 productions
//       add options like sorting alphabetically or by rating

fn main() {
    #[cfg(feature = "sdl_backend")]
    backend::sdl_backend::run_app();

    #[cfg(feature = "eframe_backend")]
    backend::eframe_backend::run_app();
}
