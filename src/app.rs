use std::borrow::Cow;
use std::sync::Arc;
use eframe::{AppCreator, egui};
use eframe::egui::panel::Side;
use eframe::egui::{Align, FontId, ImageSource, Layout, TextStyle, TopBottomPanel, Vec2, Visuals};
use eframe::egui::FontFamily::Name;
use eframe::egui::ImageSource::Uri;
use crate::config::Config;
use crate::production::Production;
use crate::themoviedb::{TheMovieDB, Width};

pub struct MovieApp {
    search: String,
    user_productions: Vec<Production>,
    search_productions: Vec<Production>,
    movie_db: TheMovieDB,
}

impl MovieApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let visuals = Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);
        Self{
            search: String::from("Search"),
            user_productions: vec![],
            search_productions: vec![],
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
                ui.heading("Find a production");
                ui.separator();
                let search_field = egui::TextEdit::singleline(&mut self.search)
                    .min_size(Vec2::new(20f32, 30f32));
                let response = ui.add(search_field);
                let pressed_enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

                if response.lost_focus() && pressed_enter{
                    self.search_productions = self.movie_db.search_production(&self.search);
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("gridder").max_col_width(300f32).min_row_height(200f32).show(ui, |ui| {
                        for movie in &self.search_productions {
                            match movie {
                                Production::Film(movie) => {
                                    if movie.poster_path.is_some() {
                                        let image_url = &self.movie_db.get_full_poster_url(
                                            movie.poster_path.clone().unwrap().as_str(),
                                            Width::W300
                                        );

                                        ui.image(Uri(Cow::from(image_url.as_str())));
                                    }

                                    let mut desc = String::from(&movie.title);
                                    desc.push('\n');
                                    desc.push_str(&movie.overview);
                                    ui.label(desc);
                                    ui.label(movie.vote_average.to_string());
                                }
                                Production::Series(show) => {
                                    if show.poster_path.is_some() {
                                        let image_url = self.movie_db.get_full_poster_url(
                                            show.poster_path.clone().unwrap().as_str(),
                                            Width::W300
                                        );

                                        ui.image(Uri(Cow::from(image_url.as_str())));
                                    }
                                    let mut desc = String::from(&show.name);
                                    desc.push('\n');
                                    desc.push_str(&show.overview);
                                    ui.label(desc);
                                    ui.label(show.vote_average.to_string());
                                }
                            }
                            ui.end_row();

                        }
                    });
                });

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
