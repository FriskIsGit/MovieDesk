mod app;
mod config;
mod credits;
mod production;
mod series_details;
mod themoviedb;

use crate::app::MovieApp;
use crate::config::Config;

// TODO: Add + in every row with a production or drag and drop to move items to the center panel
// TODO: Load images on a separate thread so it doesn't lag ui, display buffering circle(egui does it already)?
//       Perhaps make the request on a separate thread in the first place
// TODO: Temporary workaround for winit compilation time (15s)?
// TODO: Add exception handling to requests to avoid crashing the app in case something goes wrong
// TODO: Add scaling to the posters and trim long descriptions, don't artificially stretch the left
//       panel when production entries are added.
// TODO: Add a title filtering field 'Your movies' - visible only if there are at least 3 productions
//       add options like sorting alphabetically or by rating

// fn main() {
//     println!("Running!");
//     let config = Config::read_config("config.json");
//
//     let options = eframe::NativeOptions {
//         min_window_size: Some(Vec2::new(30.0, 30.0)),
//         drag_and_drop_support: true,
//         ..Default::default()
//     };
//
//     let app_creator: AppCreator = Box::new(|cc| {
//         egui_extras::install_image_loaders(&cc.egui_ctx);
//         let mut window = MovieApp::new(cc, config);
//         window.setup();
//         Box::new(window)
//     });
//
//     // Blocks the main thread.
//     let _ = eframe::run_native("App", options, app_creator);
// }

mod backend;

use backend::{ShaderVersion, DpiScaling};
use egui::FullOutput;
use sdl2::{video::GLProfile, event::Event};
use sdl2::video::SwapInterval;

use std::time::Instant;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);

    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);

    let window = video_subsystem
        .window("Demo: Egui backend for SDL2 + GL", SCREEN_WIDTH, SCREEN_HEIGHT)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    let shader_ver = ShaderVersion::Default;

    let (mut painter, mut egui_state) = backend::with_sdl2(&window, shader_ver, DpiScaling::Custom(2.0));

    let egui_context = egui::Context::default();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let enable_vsync = false;
    let quit = false;

    let start_time = Instant::now();

    println!("Running!");
    egui_extras::install_image_loaders(&egui_context);
    let config = Config::read_config("config.json");
    let mut movie_app = MovieApp::new(&egui_context, config);
    movie_app.setup();

    'running: loop {
        if enable_vsync {
            window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::VSync)
                .unwrap()
        } else {
            window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::Immediate)
                .unwrap()
        }
        
        unsafe {
            // Clear the screen to green
            gl::ClearColor(0.3, 0.6, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_context.begin_frame(egui_state.input.take());

        movie_app.update(&egui_context);

        let FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = egui_context.end_frame();

        // Process ouput
        egui_state.process_output(&window, &platform_output);

        // For default dpi scaling only, Update window when the size of resized window is very small (to avoid egui::CentralPanel distortions).
        // if egui_ctx.used_size() != painter.screen_rect.size() {
        //     println!("resized.");
        //     let _size = egui_ctx.used_size();
        //     let (w, h) = (_size.x as u32, _size.y as u32);
        //     window.set_size(w, h).unwrap();
        // }

        let paint_jobs = egui_context.tessellate(shapes);
        painter.paint_jobs(None, textures_delta, paint_jobs);
        window.gl_swap_window();

        if !repaint_after.is_zero() {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if quit {
            break;
        }
    }
}
