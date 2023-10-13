use crate::config::Config;
use crate::production::{Production, Series, Movie, UserProduction};
use crate::themoviedb::{TheMovieDB, Width};
use eframe::egui::ImageSource::Uri;
use eframe::egui::{Align, TopBottomPanel, Ui, Vec2, Visuals, Layout};
use eframe::egui;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;

pub struct MovieApp {
    // Left panel
    search: String,
    show_adult_content: bool,
    search_productions: Vec<Production>,

    // Right and center panel
    user_productions: RefCell<Vec<UserProduction>>,
    selected_user_production: Option<usize>,

    // Not a part of the layout
    movie_db: TheMovieDB,
}

impl MovieApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let visuals = Visuals::dark();
        cc.egui_ctx.set_visuals(visuals);

        Self {
            search: String::new(),
            show_adult_content: config.include_adult,
            search_productions: Vec::new(),

            user_productions: RefCell::new(Vec::new()),
            selected_user_production: None,

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
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // Put my font as last fallback for monospace:
        fonts.families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        // Tell egui to use these fonts:
        // ctx.set_fonts(fonts);
    }
}

impl eframe::App for MovieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
        self.right_panel(ctx);
    }
}

const MOVIE_URL: &str = "https://www.themoviedb.org/movie/";
const TV_URL: &str = "https://www.themoviedb.org/tv/";

impl MovieApp {
    fn left_panel(&mut self, ctx: &egui::Context) {
        let left = egui::SidePanel::left("left_panel");
        left.resizable(true).show(ctx, |ui| {
            let mut search_triggered = false;

            ui.heading("Find a production");
            ui.separator();

            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                let search_field = egui::TextEdit::singleline(&mut self.search)
                    .min_size(Vec2::new(20.0, 0.0))
                    .hint_text("Search");

                let response = ui.add(search_field);
                let pressed_enter = ui.input(|i| i.key_pressed(egui::Key::Enter));

                if response.lost_focus() && pressed_enter {
                    self.search_productions = self.movie_db.search_production(&self.search);
                    search_triggered = true;
                }
            });

            ui.add_space(5.0);

            ui.checkbox(&mut self.show_adult_content, "Show adult content");
            ui.separator();

            self.production_grid(ui, search_triggered);
            ui.separator();
        });
    }

    fn central_panel(&mut self, ctx: &egui::Context) {
        let center = egui::CentralPanel::default();
        center.show(ctx, |ui| {
            ui.heading("Your movies!");
            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("grid_center").show(ui, |ui| {
                    let entries = self.user_productions.borrow();
                    for (i, entry) in entries.iter().enumerate() {
                        // NOTE: This is a placeholder. You should be able to click on an entire
                        //       grid entry and then the whole thing should be highlighted.
                        if let Some(index) = self.selected_user_production {
                            let mut checked = index == i;
                            if ui.checkbox(&mut checked, "").clicked() {
                                self.selected_user_production = Some(i);
                            }
                        } else {
                            let mut checked = false;
                            if ui.checkbox(&mut checked, "").clicked() {
                                self.selected_user_production = Some(i);
                            }
                        }

                        match &entry.production {
                            Production::Movie(movie) => {
                                if movie.poster_path.is_some() {
                                    let image_url = TheMovieDB::get_full_poster_url(
                                        movie.poster_path.to_owned().unwrap().as_str(),
                                        Width::W300,
                                    );

                                    ui.image(Uri(Cow::from(image_url.as_str())));
                                    ui.heading(&movie.title);
                                }
                            }
                            Production::Series(show) => {
                                if show.poster_path.is_some() {
                                    let image_url = TheMovieDB::get_full_poster_url(
                                        show.poster_path.to_owned().unwrap().as_str(),
                                        Width::W300,
                                    );

                                    ui.image(Uri(Cow::from(image_url.as_str())));
                                    ui.heading(&show.name);
                                }
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
            ui.heading("Selected production");
            ui.separator();

            let Some(index) = self.selected_user_production else {
                ui.add_space(10.0);
                ui.label("Currently nothing is selected ._.");
                return;
            };
            
            let mut user_productions = self.user_productions.borrow_mut();
            let Some(entry) = user_productions.get_mut(index) else {
                ui.add_space(10.0);
                ui.label("Currently nothing is selected ._.");
                return;
            };

            // Looks kinda wack, but we can change it later...
            match &entry.production {
                Production::Movie(movie) => {
                    ui.heading(&movie.title);

                    if let Some(poster) = &movie.poster_path {
                        let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                        let image = egui::Image::new(Uri(image_url.into()));
                        ui.add_sized([100.0, 100.0], image);
                    }
                }
                Production::Series(series) => {
                    ui.heading(&series.name);

                    if let Some(poster) = &series.poster_path {
                        let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                        let image = egui::Image::new(Uri(image_url.into()));
                        ui.add_sized([100.0, 100.0], image);
                    }
                }
            }

            ui.separator();
            ui.add_space(8.0);

            ui.label("Your rating:");
            ui.horizontal(|ui| {
                // Make this a custom button/slider thing where you click on stars to select rating?
                // ⭐⭐⭐⭐⭐
                ui.add(egui::DragValue::new(&mut entry.user_rating).speed(0.1));
                ui.label("/ 10")
            });

            ui.add_space(8.0);

            ui.label("Your notes:");
            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                ui.text_edit_multiline(&mut entry.user_note);
            });
        });

    }

    // Could be used for some toolbar logic at the top of the layout.
    // | File | View | Settings | Help | Info | ... etc.
    // Just like many popular programs.
    fn top_panel(&self, ctx: &egui::Context) {
        let top = TopBottomPanel::top("top_panel");
        top.resizable(true).show(ctx, |_| {});
    }

    fn draw_movie_entry(&self, ui: &mut Ui, movie: &Movie) {
        if movie.adult && !self.show_adult_content {
            return;
        }

        ui.horizontal(|ui| {
            if let Some(poster) = &movie.poster_path {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);

                let image = egui::Image::new(Uri(image_url.into()));
                let poster = ui.add_sized([60.0, 100.0], image)
                    .interact(egui::Sense::click());
                poster.context_menu(|ui| {
                    if ui.button("Add movie").clicked() {
                        let mut user_productions = self.user_productions.borrow_mut();
                        let exists = user_productions.iter().any(|entry| {
                            let Production::Movie(user_movie) = &entry.production else { return false };
                            user_movie.id == movie.id
                        });

                        if !exists {
                            let new_data = UserProduction {
                                production: Production::Movie(movie.clone()),
                                user_note: String::new(),
                                user_rating: 0.0,
                            };
                            user_productions.push(new_data);
                        }
                        ui.close_menu()
                    }

                    if ui.button("Open in tmdb").clicked() {
                        let mut path = String::from(MOVIE_URL);
                        path.push_str(movie.id.to_string().as_str());
                        let browser = &self.movie_db.config.browser_name;
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Open in imdb").clicked() {
                        let path = format!("https://www.imdb.com/find/?q={}", movie.title);
                        let browser = &self.movie_db.config.browser_name;
                        //use External IDs (movie endpoint)
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Download poster").clicked(){
                        if movie.poster_path.is_some() {
                            let poster = movie.poster_path.as_ref().unwrap().as_str();
                            let resource = TheMovieDB::get_full_poster_url(poster, Width::ORIGINAL);
                            let bytes = self.movie_db.download_resource(resource.as_str());
                            let mut file = File::create(&poster[1..]).expect("Unable to create file");
                            // Write a slice of bytes to the file
                            file.write_all(&bytes).unwrap();
                        }
                    }

                    if ui.button("Close menu").clicked() {
                        ui.close_menu();
                    }
                });
            }

            ui.vertical(|ui| {
                ui.add_space(10.0);
                ui.heading(&movie.title);
                ui.add_space(8.0);
                ui.label(format!("Rating: {} / 10", movie.vote_average));
                ui.add_space(4.0);
                ui.label(format!("Release date: {}", movie.release_date));
            });
        });

        ui.add_space(5.0);

        if movie.overview.len() > 200 {
            // NOTE: This is really bad! We should cache the output of the format to not call
            // it every single frame. We should also not take the slice here because we can
            // panic since strings are UTF-8 and this take bytes.
            //                                 vvvvvvvvvvvvvvvvvvvvvv
            let description = format!("{}...", &movie.overview[..200].trim());
            ui.label(description);
        } else {
            ui.label(&movie.overview);
        };

        ui.separator();
    }

    fn draw_series_entry(&self, ui: &mut Ui, series: &Series) {
        if series.adult && !self.show_adult_content {
            return;
        }

        ui.horizontal(|ui| {
            if let Some(poster) = &series.poster_path {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);

                let image = egui::Image::new(Uri(image_url.into()));
                let poster = ui.add_sized([60.0, 100.0], image)
                    .interact(egui::Sense::click());

                poster.context_menu(|ui| {
                    if ui.button("Add show").clicked() {
                        let mut user_productions = self.user_productions.borrow_mut();
                        let exists = user_productions.iter().any(|entry| {
                            let Production::Series(user_show) = &entry.production else { return false };
                            user_show.id == series.id
                        });

                        if !exists {
                            let new_data = UserProduction {
                                production: Production::Series(series.clone()),
                                user_note: String::new(),
                                user_rating: 0.0,
                            };
                            user_productions.push(new_data);
                        }
                        ui.close_menu()
                    }

                    if ui.button("Open in tmdb").clicked() {
                        let mut path = String::from(TV_URL);
                        path.push_str(series.id.to_string().as_str());
                        let browser = &self.movie_db.config.browser_name;
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Open in imdb").clicked() {
                        let path = format!("https://www.imdb.com/find/?q={}", series.name);
                        let browser = &self.movie_db.config.browser_name;
                        //its buggy open in tmdb instead?
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Download poster").clicked(){
                        if series.poster_path.is_some() {
                            let poster = series.poster_path.as_ref().unwrap().as_str();
                            let resource = TheMovieDB::get_full_poster_url(poster, Width::ORIGINAL);
                            let bytes = self.movie_db.download_resource(resource.as_str());
                            let mut file = File::create(&poster[1..]).expect("Unable to create file");
                            // Write a slice of bytes to the file
                            file.write_all(&bytes).unwrap();
                        }
                    }

                    if ui.button("Close menu").clicked() {
                        ui.close_menu();
                    }
                });
            }

            ui.vertical(|ui| {
                ui.add_space(10.0);
                ui.heading(&series.name);
                ui.add_space(8.0);
                ui.label(format!("Rating: {} / 10", series.vote_average));
                ui.add_space(4.0);
                ui.label(format!("First air date: {}", series.first_air_date));
            });
        });

        ui.add_space(5.0);

        if series.overview.len() > 200 {
            // NOTE: This is really bad! We should cache the output of the format to not call
            // it every single frame. We should also not take the slice here because we can
            // panic since strings are UTF-8 and this take bytes.
            //                                 vvvvvvvvvvvvvvvvvvvvvv
            let description = format!("{}...", &series.overview[..200]);
            ui.label(description);
        } else {
            ui.label(&series.overview);
        };

        ui.separator();
    }

    fn production_grid(&mut self, ui: &mut Ui, searched: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if searched {
                ui.scroll_to_cursor(Some(Align::Min));
            }

            for movie in self.search_productions.iter() {
                match movie {
                    Production::Movie(movie) => self.draw_movie_entry(ui, movie),
                    Production::Series(series) => self.draw_series_entry(ui, series),
                }
            }
        });
    }
}
