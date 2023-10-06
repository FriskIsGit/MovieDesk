use std::sync::Arc;
use eframe::{AppCreator, egui};
use eframe::egui::panel::Side;
use eframe::egui::{Align, FontId, Layout, TextStyle, TopBottomPanel, Vec2, Visuals};
use eframe::egui::FontFamily::Name;
use crate::config::Config;
use crate::themoviedb::{Movie, TheMovieDB};

pub struct MovieApp {
    search: String,
    user_movies: Vec<Movie>,
    movie_db: TheMovieDB,
}

impl MovieApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let visuals = Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);
        Self{
            search: String::from("Search"),
            user_movies: vec![],
            movie_db: TheMovieDB::new(config),
        }
    }
    pub fn setup(&mut self) {
        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        /*fonts.font_data.insert(
            "my_font".to_owned(),
            egui::FontData::from_static(include_bytes!("/fonts/Hack-Regular.ttf")),
        );*/

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        // Tell egui to use these fonts:
        //ctx.set_fonts(fonts);
    }

}

impl eframe::App for MovieApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.search_panel(&ctx);
        self.right_panel(&ctx);
        self.central_panel(&ctx);
    }
}

impl MovieApp {
    fn search_panel(&mut self, ctx: &egui::Context) {
        let left = egui::SidePanel::left("search_panel");
        left.resizable(true)
            .show(ctx, |ui| {
                ui.heading("Find a movie");
                ui.separator();
                let search_field = egui::TextEdit::singleline(&mut self.search)
                    .min_size(Vec2::new(10f32, 10f32));
                let response = ui.add(search_field);
                let pressed_enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if response.lost_focus() && pressed_enter{
                    self.movie_db.search_movie(&self.search);
                    println!("{}", &self.search);
                }

                ui.label("Kihau waz here...");
                ui.image(egui::include_image!("../res/test.png"));
            });
    }

    fn right_panel(&self, ctx: &egui::Context) {
        let right = egui::SidePanel::right("right_panel");
        right.show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                let _ = ui.button("Look at me, I am becoming so wide!");
                ui.label("TestLabel");
            });
            egui::ScrollArea::vertical().show(ui, |ui| {});
        });
    }

    fn central_panel(&self, ctx: &egui::Context) {
        let center = egui::CentralPanel::default();
        center.show(ctx, |ui| {
            ui.heading("Your movies!");
            ui.separator()
        });
    }

    fn top_panel(&mut self, ctx: &egui::Context) {
        let left = TopBottomPanel::top("top_panel");
        left.resizable(true)
            .show(ctx, |ui| {
            });
    }
}
