use crate::config::Config;
use crate::production::{Production, Series, Movie};
use crate::themoviedb::{TheMovieDB, Width};
use eframe::egui::ImageSource::Uri;
use eframe::egui::{Align, Layout, TopBottomPanel, Ui, Vec2, Visuals};
use eframe::egui;
use std::borrow::Cow;
use std::cell::RefCell;

pub struct MovieApp {
    show_adult_content: bool,
    search: String,
    user_productions: RefCell<Vec<Production>>,
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
            user_productions: RefCell::new(vec![]),
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
        self.search_panel(ctx);
        self.right_panel(ctx);
        self.central_panel(ctx);
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

    fn central_panel(&self, ctx: &egui::Context) {
        let center = egui::CentralPanel::default();
        center.show(ctx, |ui| {
            ui.heading("Your movies!");
            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("grid_center").show(ui, |ui| {
                    for production in self.user_productions.borrow().iter() {
                        match production {
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

    fn top_panel(&self, ctx: &egui::Context) {
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

            poster.context_menu(|ui| {
                if ui.button("Add movie").clicked() {
                    let mut user_productions = self.user_productions.borrow_mut();
                    let exists = user_productions.iter().any(|prod| {
                        let Production::Movie(user_movie) = prod else { return false };
                        user_movie.id == movie.id
                    });

                    if !exists {
                        user_productions.push(Production::Movie(movie.clone()));
                    }
                    ui.close_menu()
                }
                //this is slower than I expected
                if ui.button("Open in imdb").clicked() {
                    let mut path = String::from("https://www.imdb.com/find/?q=");
                    path.push_str(&movie.title);
                    //its buggy open in tmdb instead?
                    let _ = open::with(path, &self.movie_db.config.browser_name); //does it get encoded or do we need to encode it?
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
            ui.add_space(8.0);
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
        });
    }

    fn add_show_entry(&self, ui: &mut Ui, show: &Series) {
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

            poster.context_menu(|ui| {
                if ui.button("Add show").clicked() {
                    let mut user_productions = self.user_productions.borrow_mut();
                    let exists = user_productions.iter().any(|prod| {
                        let Production::Series(user_show) = prod else { return false };
                        user_show.id == show.id
                    });

                    if !exists {
                        user_productions.push(Production::Series(show.clone()));
                    }
                    ui.close_menu()
                }
                //this is slower than I expected
                if ui.button("Open in imdb").clicked() {
                    let mut path = String::from("https://www.imdb.com/find/?q=");
                    path.push_str(&show.name);
                    //its buggy open in tmdb instead?
                    let _ = open::with(path, &self.movie_db.config.browser_name); //does it get encoded or do we need to encode it?
                    ui.close_menu()
                }

                if ui.button("Close menu").clicked() {
                    ui.close_menu();
                }
            });

        }

        ui.vertical(|ui| {
            ui.add_space(10.0);
            ui.heading(&show.name);
            ui.add_space(8.0);
            ui.label(format!("Rating: {} / 10", show.vote_average));
            ui.add_space(4.0);
            ui.label(format!("First air date: {}", show.first_air_date));
            ui.add_space(8.0);
            if show.overview.len() > 200 {
                // NOTE: This is really bad! We should cache the output of the format to not call
                // it every single frame. We should also not take the slice here because we can
                // panic since strings are UTF-8 and this take bytes.
                //                                 vvvvvvvvvvvvvvvvvvvvvv
                let description = format!("{}...", &show.overview[..200]);
                ui.label(description);
            } else {
                ui.label(&show.overview);
            };
        });
    }

    fn production_grid(&mut self, ui: &mut Ui, searched: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if searched {
                ui.scroll_to_cursor(Some(Align::Min));
            }

            egui::Grid::new("grid_left").max_col_width(200.0).min_row_height(200.0).show(ui, |ui| {
                for movie in self.search_productions.iter() {
                    match movie {
                        Production::Movie(movie) => self.add_film_entry(ui, movie),
                        Production::Series(show) => self.add_show_entry(ui, show),
                    }
                    ui.end_row();
                }
            });
        });
    }
}
