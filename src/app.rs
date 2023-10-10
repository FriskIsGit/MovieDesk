use crate::config::Config;
use crate::production::{Production, TVShow, Movie};
use crate::themoviedb::{TheMovieDB, Width};
use eframe::egui::ImageSource::Uri;
use eframe::egui::{Align, Layout, TopBottomPanel, Ui, Vec2, Visuals};
use eframe::egui;
use std::borrow::Cow;

pub struct MovieApp {
    show_adult_content: bool,
    search: String,
    user_productions: Vec<Production>,
    search_productions: Vec<Production>,
    movie_db: TheMovieDB,
}

impl MovieApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let visuals = Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);
        Self {
            show_adult_content: config.include_adult,
            search: String::new(),
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
        // fonts.font_data.insert(
        //     "my_font".to_owned(),
        //     egui::FontData::from_static(include_bytes!("/fonts/Hack-Regular.ttf")),
        // );

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
        // ctx.set_fonts(fonts);
    }
}

impl eframe::App for MovieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.search_panel(&ctx);
        self.right_panel(&ctx);
        self.central_panel(&ctx);
    }
}

impl MovieApp {
    fn search_panel(&mut self, ctx: &egui::Context) {
        let left = egui::SidePanel::left("search_panel");
        left.resizable(true).show(ctx, |ui| {
            ui.heading("Find a production");
            ui.separator();
            let search_field = egui::TextEdit::singleline(&mut self.search)
                .min_size(Vec2::new(20.0, 0.0))
                .hint_text("Search");

            let response = ui.add(search_field);
            let pressed_enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

            let mut search_triggered = false;

            if response.lost_focus() && pressed_enter {
                self.search_productions = self.movie_db.search_production(&self.search);
                search_triggered = true;
            }

            ui.add_space(5.0);

            ui.checkbox(&mut self.show_adult_content, "Show adult content");
            ui.separator();

            self.production_grid(ui, search_triggered);
            ui.separator();
        });
    }

    fn right_panel(&self, ctx: &egui::Context) {
        let right = egui::SidePanel::right("right_panel");
        right.show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                let _ = ui.button("Look at me, I am becoming so wide!");
                ui.label("TestLabel");
            });
            egui::ScrollArea::vertical().show(ui, |_| {});
        });
    }

    fn central_panel(&mut self, ctx: &egui::Context) {
        let center = egui::CentralPanel::default();
        center.show(ctx, |ui| {
            ui.heading("Your movies!");
            ui.separator();
        });
    }

    fn top_panel(&mut self, ctx: &egui::Context) {
        let left = TopBottomPanel::top("top_panel");
        left.resizable(true).show(ctx, |_| {});
    }

    fn add_film_entry(&self, ui: &mut Ui, movie: &Movie) {
        if movie.adult && !self.show_adult_content {
            return;
        }

        if movie.poster_path.is_some() {
            let image_url = TheMovieDB::get_full_poster_url(
                movie.poster_path.to_owned().unwrap().as_str(),
                Width::W300,
            );

            let poster = ui
                .image(Uri(Cow::from(image_url.as_str())))
                .interact(egui::Sense::click());

            if poster.clicked() {
                println!("CLICKED ON: {}", movie.title);
            }
        }

        let mut desc = String::from(&movie.title);
        desc.push('\n');
        desc.push_str(&movie.overview);
        ui.label(desc);
        ui.label(movie.vote_average.to_string());
    }

    fn add_show_entry(&self, ui: &mut Ui, show: &TVShow) {
        if show.adult && !self.show_adult_content {
            return;
        }

        if show.poster_path.is_some() {
            let image_url = TheMovieDB::get_full_poster_url(
                show.poster_path.to_owned().unwrap().as_str(),
                Width::W300,
            );

            let poster = ui
                .image(Uri(Cow::from(image_url.as_str())))
                .interact(egui::Sense::click());

            if poster.clicked() {
                println!("CLICKED ON: {}", show.name);
                let show_details = self.movie_db.get_show_details(show.id);
                let season_details = self.movie_db.get_season_details(
                    show.id,
                    show_details.number_of_seasons,
                );
                println!("season details {:?}", season_details);
            }
        }
        let mut desc = String::from(&show.name);
        desc.push('\n');
        desc.push_str(&show.overview);
        ui.label(desc);
        ui.label(show.vote_average.to_string());
    }

    fn production_grid(&self, ui: &mut Ui, searched: bool) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if searched {
                ui.scroll_to_cursor(Some(Align::Min));
            }

            egui::Grid::new("gridder")
                .max_col_width(180.0)
                .min_row_height(200.0)
                .show(ui, |ui| {
                    for movie in self.search_productions.iter() {
                        match movie {
                            Production::Film(movie) => self.add_film_entry(ui, movie),
                            Production::Series(show) => self.add_show_entry(ui, show),
                        }
                        ui.end_row();
                    }
                });
        });
    }
}
