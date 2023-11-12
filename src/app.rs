use crate::config::Config;
use crate::jobs::Job;
use crate::production::{Movie, Production, Series, UserMovie, UserSeries};
use crate::series_details::{SeasonDetails, SeriesDetails};
use crate::themoviedb::{TheMovieDB, Width};
use crate::view::{SeriesView, MovieView, TrailersView};

use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::production;
use egui::ImageSource::Uri;
use egui::{Align, Layout, TopBottomPanel, Ui, Vec2, Visuals};

pub struct MovieApp {
    // Left panel
    search: String,
    show_adult_content: bool,

    search_productions: Option<Rc<[Production]>>,
    search_cache: HashMap<String, Rc<[Production]>>,
    fetch_productions_job: Job<(String, Vec<Production>)>,

    // Right and center panel
    user_movies: Vec<UserMovie>,
    user_series: Vec<UserSeries>,
    selected_user_movie: Option<usize>,
    selected_user_series: Option<usize>,

    // Notes
    series_details_job: Job<SeriesDetails>,
    season_details_job: Job<SeasonDetails>,
    series_details: Option<SeriesDetails>,
    season_details: Option<SeasonDetails>,
    selected_season: Option<u32>,
    selected_episode: Option<u32>,

    // View states
    series_view: SeriesView,
    movie_view: MovieView,
    trailers_view: TrailersView,

    // Not a part of the layout
    movie_db: TheMovieDB,
    config: Config,
}

impl MovieApp {
    pub fn new(ctx: &egui::Context, mut config: Config) -> Self {
        let visuals = Visuals::dark();
        ctx.set_visuals(visuals);

        // Implement dynamic scale changing?
        ctx.set_pixels_per_point(1.5);

        // YOINK! We are not going to need the this here anymore.
        // The api key is only used by TheMovieDB.
        let key = std::mem::take(&mut config.api_key);

        let movie_db = TheMovieDB::new(key, config.enable_cache);
        Self {
            search: String::new(),
            show_adult_content: config.include_adult,
            search_productions: None,
            search_cache: HashMap::default(),
            fetch_productions_job: Job::Empty,

            user_movies: Vec::new(),
            user_series: Vec::new(),
            selected_user_movie: None,
            selected_user_series: None,
            series_details_job: Job::Empty,
            season_details_job: Job::Empty,
            series_details: None,
            season_details: None,
            selected_season: None,
            selected_episode: None,

            series_view: SeriesView::new(),
            movie_view: MovieView::new(),
            trailers_view: TrailersView::new(),

            movie_db,
            config,
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

    pub fn render(&mut self, ctx: &egui::Context) {
        self.series_view.draw(ctx, &self.movie_db);
        self.movie_view.draw(ctx, &self.movie_db);
        self.trailers_view.draw(ctx);

        self.top_panel(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.central_panel(ctx);
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
                    match self.search_cache.get(&self.search).cloned() {
                        None => {
                            self.fetch_productions_job = self.movie_db.search_production(self.search.clone());
                            search_triggered = true;
                        }
                        res @ Some(_) => self.search_productions = res,
                    }
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
                    // NOTE: This is a placeholder. You should be able to click on an entire
                    //       grid entry and then the whole thing should be highlighted.
                    let movie_entries = &self.user_movies;
                    for (i, entry) in movie_entries.iter().enumerate() {
                        let movie = &entry.movie;
                        match self.selected_user_movie {
                            Some(index) => {
                                let mut checked = index == i;
                                if ui.checkbox(&mut checked, "").clicked() {
                                    self.selected_user_movie = Some(i);
                                    self.selected_user_series = None;
                                }
                            }
                            None => {
                                if ui.checkbox(&mut false, "").clicked() {
                                    self.selected_user_movie = Some(i);
                                    self.selected_user_series = None;
                                }
                            }
                        }
                        if movie.poster_path.is_some() {
                            let image_url =
                                TheMovieDB::get_full_poster_url(movie.poster_path.as_ref().unwrap(), Width::W300);

                            ui.image(Uri(image_url.into()));
                            ui.heading(&movie.title);
                        }
                        ui.end_row();
                    }

                    let series_entries = &self.user_series;
                    for (i, entry) in series_entries.iter().enumerate() {
                        let series = &entry.series;

                        match self.selected_user_series {
                            Some(index) => {
                                let mut checked = index == i;
                                if ui.checkbox(&mut checked, "").clicked() {
                                    self.selected_user_series = Some(i);
                                    self.selected_user_movie = None;
                                    self.series_details_job = self.movie_db.get_series_details(series.id);
                                    self.selected_episode = None;
                                    self.selected_season = None;
                                }
                            }
                            None => {
                                if ui.checkbox(&mut false, "").clicked() {
                                    self.selected_user_series = Some(i);
                                    self.selected_user_movie = None;
                                    // start two jobs: fetch seasons, fetch episodes for each series
                                    // now since caching is internal to app.rs then im gonna bother w that
                                    self.series_details_job = self.movie_db.get_series_details(series.id);
                                    self.selected_episode = None;
                                    self.selected_season = None;
                                }
                            }
                        }

                        if series.poster_path.is_some() {
                            let image_url =
                                TheMovieDB::get_full_poster_url(series.poster_path.as_ref().unwrap(), Width::W300);

                            ui.image(Uri(image_url.into()));
                            ui.heading(&series.name);
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }

    fn right_panel(&mut self, ctx: &egui::Context) {
        let right = egui::SidePanel::right("right_panel");
        // This needs a lot of changes
        right.show(ctx, |ui| {
            let heading;
            if self.selected_user_movie.is_none() && self.selected_user_series.is_none() {
                ui.add_space(10.0);
                ui.label("Currently nothing is selected ._.");
                return;
            }

            let is_movie;
            match self.selected_user_movie {
                Some(_) => {
                    heading = "Selected movie";
                    is_movie = true;
                }
                None => {
                    // assuming we reset selected_user_movie
                    heading = "Selected series";
                    is_movie = false;
                }
            }
            ui.heading(heading);
            ui.separator();

            // let mut user_productions = self.user_productions.borrow_mut();
            /*let Some(entry) = self.user_movies.get_mut(index) else {
                ui.add_space(10.0);
                ui.label("Currently nothing is selected ._.");
                return;
            };*/
            if is_movie {
                let Some(user_movie) = self.user_movies.get_mut(self.selected_user_movie.unwrap()) else {
                    return;
                };
                let movie = &user_movie.movie;
                ui.heading(&movie.title);

                if let Some(poster) = &movie.poster_path {
                    let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                    let image = egui::Image::new(Uri(image_url.into()));
                    ui.add_sized([100.0, 100.0], image);
                }
            } else {
                let Some(user_series) = self.user_series.get_mut(self.selected_user_series.unwrap()) else {
                    return;
                };
                let series = &user_series.series;
                ui.heading(&series.name);

                if let Some(poster) = &series.poster_path {
                    let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                    let image = egui::Image::new(Uri(image_url.into()));
                    ui.add_sized([100.0, 100.0], image);
                }

                if let Some(details) = self.series_details_job.poll_owned() {
                    self.series_details = Some(details);
                }
                if let Some(details) = &self.series_details {
                    let display = if self.selected_season.is_some() {
                        format!("S{}", self.selected_season.unwrap())
                    } else {
                        "None".to_string()
                    };
                    let before_render_season = self.selected_season.unwrap_or(0);
                    egui::ComboBox::from_label("Select season!")
                        .selected_text(display)
                        .show_ui(ui, |ui| {
                            for i in 1..=details.number_of_seasons {
                                ui.selectable_value(&mut self.selected_season, Some(i), format!("S{}", i));
                            }
                            ui.selectable_value(&mut self.selected_season, None, "None");
                        });

                    let after_render_season = self.selected_season.unwrap_or(0);
                    if before_render_season != after_render_season {
                        self.season_details_job = self.movie_db.get_season_details(series.id, after_render_season);
                        self.selected_episode = None;
                    }
                    if self.selected_season.is_some() {
                        if let Some(season_details) = self.season_details_job.poll_owned() {
                            self.season_details = Some(season_details);
                        }
                        if let Some(season_details) = &self.season_details {
                            let display = if self.selected_episode.is_some() {
                                format!("EP{}", self.selected_episode.unwrap())
                            } else {
                                "None".to_string()
                            };
                            let all_episodes = season_details.episodes.len();
                            egui::ComboBox::from_label("Select episode!")
                                .selected_text(display)
                                .show_ui(ui, |ui| {
                                    for i in 1..=all_episodes {
                                        ui.selectable_value(
                                            &mut self.selected_episode,
                                            Some(i as u32),
                                            format!("EP{}", i),
                                        );
                                    }
                                    ui.selectable_value(&mut self.selected_episode, None, "None");
                                });
                        }
                    }
                }
            }

            ui.separator();
            ui.add_space(8.0);

            // lots of duplicates
            ui.label("Your rating:");
            let user_movie;
            let user_series;
            if is_movie {
                user_movie = self.user_movies.get_mut(self.selected_user_movie.unwrap()).unwrap();
                ui.horizontal(|ui| {
                    // Make this a custom button/slider thing where you click on stars to select rating?
                    // ⭐⭐⭐⭐⭐
                    ui.add(
                        egui::DragValue::new(&mut user_movie.user_rating)
                            .speed(0.1)
                            .clamp_range(RangeInclusive::new(0.0, 10.0)),
                    );
                    ui.label("/ 10")
                });
                ui.add_space(8.0);
                ui.label("Your notes:");
                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                    ui.text_edit_multiline(&mut user_movie.note);
                });
            } else {
                user_series = self.user_series.get_mut(self.selected_user_series.unwrap()).unwrap();
                ui.horizontal(|ui| {
                    // Make this a custom button/slider thing where you click on stars to select rating?
                    // ⭐⭐⭐⭐⭐
                    ui.add(
                        egui::DragValue::new(&mut user_series.user_rating)
                            .speed(0.1)
                            .clamp_range(RangeInclusive::new(0.0, 10.0)),
                    );
                    ui.label("/ 10")
                });
                ui.add_space(8.0);
                if let Some(episode_num) = self.selected_episode {
                    let season_num = self.selected_season.unwrap();
                    let series_details = self.series_details.as_ref().unwrap();
                    // we shouldn't ensure length every frame but at the same time we shouldn't
                    // allocate all of it because series can be very big and we save space in json (read/write)
                    user_series.ensure_seasons(series_details.number_of_seasons as usize);
                    ui.label(format!("Episode {} notes:", episode_num));
                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                        let season_notes = &mut user_series.season_notes[season_num as usize - 1];
                        season_notes.ensure_episodes(series_details.number_of_episodes as usize);
                        ui.text_edit_multiline(&mut season_notes.episode_notes[episode_num as usize - 1]);
                    });
                    return;
                }
                if let Some(season_num) = self.selected_season {
                    let series_details = self.series_details.as_ref().unwrap();
                    // we shouldn't ensure length every frame but at the same time we shouldn't
                    // allocate all of it because series can be very big and we save space in json (read/write)
                    user_series.ensure_seasons(series_details.number_of_seasons as usize);
                    ui.label(format!("Season {} notes:", season_num));
                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                        ui.text_edit_multiline(&mut user_series.season_notes[season_num as usize - 1].note);
                    });
                    return;
                }
                ui.label("Your notes:");
                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                    ui.text_edit_multiline(&mut user_series.note);
                });
            }
        });
    }

    // Could be used for some toolbar logic at the top of the layout.
    // | File | View | Settings | Help | Info | ... etc.
    // Just like many popular programs.
    fn top_panel(&mut self, ctx: &egui::Context) {
        let top = TopBottomPanel::top("top_panel");
        top.resizable(true).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    // display success/failure message somewhere once finished below?
                    if ui.button("Save data").clicked() && (!self.user_movies.is_empty() || !self.user_movies.is_empty()) {
                        let outcome = production::serialize_user_productions(&self.user_series, &self.user_movies);
                        if outcome.is_err() {
                            eprintln!("{}", outcome.unwrap_err())
                        }
                    }
                    if ui.button("Load data").clicked() {
                        let outcome = production::deserialize_user_productions(None);
                        match outcome {
                            Ok(user_prods) => {
                                self.user_series = user_prods.0;
                                self.user_movies = user_prods.1;
                            }
                            Err(msg) => eprintln!("{}", msg)
                        }
                    }
                    if ui.button("Load data from file").clicked() {}
                });

                ui.menu_button("View", |ui| {
                    // why is this so laggy?
                    if ui.button("PPP +0.01").clicked() {
                        ctx.set_pixels_per_point(ctx.pixels_per_point() + 0.01);
                    }
                    if ui.button("PPP -0.01").clicked() {
                        ctx.set_pixels_per_point(ctx.pixels_per_point() - 0.01);
                    }
                });

                ui.menu_button("Settings", |ui| {
                    /* The settings menu:
                        - [ ] Auto-save
                            - [ ] Enable/Disable
                            - [ ] Update interval
                        - [ ] Sync
                            - [ ] Sync to server
                            - [ ] Sync from server
                        - [ ] Enable/Disable local caching
                        - [ ] Set tmdb token
                        - [ ] Set default browser
                    */

                    if ui.button("Auto-save").clicked() {}
                    if ui.button("Enable caching").clicked() {}
                    if ui.button("Set TMDB token").clicked() {}
                    if ui.button("Set default browser").clicked() {}
                    if ui.button("Sync").clicked() {}
                });

                ui.menu_button("About", |_| {});
                ui.menu_button("License", |_| {});
            });
        });
    }

    fn draw_movie_entry(&mut self, ui: &mut Ui, movie: &Movie) {
        if movie.adult && !self.show_adult_content {
            return;
        }

        ui.horizontal(|ui| {
            if let Some(poster) = movie.poster_path.as_ref() {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);

                let image = egui::Image::new(Uri(image_url.into()));
                let poster = ui.add_sized([60.0, 100.0], image).interact(egui::Sense::click());
                poster.context_menu(|ui| {
                    if ui.button("Add movie").clicked() {
                        // let mut user_productions = self.user_productions.borrow_mut();
                        let exists = self
                            .user_movies
                            .iter()
                            .any(|user_movie| user_movie.movie.id == movie.id);

                        if !exists {
                            let new_data = UserMovie {
                                movie: movie.clone(),
                                note: String::new(),
                                user_rating: 0.0,
                            };
                            self.user_movies.push(new_data);
                        }
                        ui.close_menu()
                    }
                    //change name?: xpanded view, about, more, view seasons, view more, view details,
                    if ui.button("More details").clicked() {
                        self.movie_view.set_movie(movie.clone(), &self.movie_db);
                        ui.close_menu();
                    }

                    if ui.button("Open in TMDB").clicked() {
                        let mut path = String::from(MOVIE_URL);
                        path.push_str(movie.id.to_string().as_str());
                        let browser = &self.config.browser_name;
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Open in IMDB").clicked() {
                        let url = self.movie_db.get_imdb_url(Production::Movie(movie.to_owned()));
                        let browser = &self.config.browser_name;
                        let _ = open::with_in_background(url, browser);
                    }

                    if ui.button("Fetch trailers").clicked() {
                        let trailers = self.movie_db.get_movie_trailers(movie.id);
                        self.trailers_view.set_content(movie.title.to_owned(), trailers);
                    }

                    if ui.button("Download poster").clicked() && movie.poster_path.is_some() {
                        let poster = movie.poster_path.as_ref().unwrap();
                        let resource = TheMovieDB::get_full_poster_url(poster, Width::ORIGINAL);
                        self.movie_db.download_poster(&resource, &poster[1..]);
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
            //       it every single frame. We should also not take the slice here because we can
            //       panic since strings are UTF-8 and this takes bytes.
            //                                 vvvvvvvvvvvvvvvvvvvvvv
            let description = format!("{}...", &movie.overview[..200].trim());
            ui.label(description);
        } else {
            ui.label(&movie.overview);
        };

        ui.separator();
    }

    fn draw_series_entry(&mut self, ui: &mut Ui, series: &Series) {
        if series.adult && !self.show_adult_content {
            return;
        }

        ui.horizontal(|ui| {
            if let Some(poster) = &series.poster_path {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);

                let image = egui::Image::new(Uri(image_url.into()));
                let poster = ui.add_sized([60.0, 100.0], image).interact(egui::Sense::click());

                poster.context_menu(|ui| {
                    if ui.button("Add series").clicked() {
                        // let mut user_productions = self.user_productions.borrow_mut();
                        let exists = self
                            .user_series
                            .iter()
                            .any(|user_series| user_series.series.id == series.id);

                        if !exists {
                            let new_data = UserSeries {
                                series: series.clone(),
                                note: String::new(),
                                user_rating: 0.0,
                                season_notes: Vec::new(),
                            };
                            self.user_series.push(new_data);
                        }
                        ui.close_menu()
                    }

                    if ui.button("More series details").clicked() {
                        self.series_view.set_series(series.clone(), &self.movie_db);
                        ui.close_menu();
                    }

                    if ui.button("Open in TMDB").clicked() {
                        let mut path = String::from(TV_URL);
                        path.push_str(series.id.to_string().as_str());
                        let browser = &self.config.browser_name;
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Open in IMDB").clicked() {
                        let url = self.movie_db.get_imdb_url(Production::Series(series.to_owned()));
                        let browser = &self.config.browser_name;
                        let _ = open::with_in_background(url, browser);
                    }

                    if ui.button("Fetch trailers").clicked() {
                        let trailers = self.movie_db.get_series_trailers(series.id);
                        self.trailers_view.set_content(series.name.to_owned(), trailers);
                    }

                    if ui.button("Download poster").clicked() && series.poster_path.is_some() {
                        let poster = series.poster_path.clone().unwrap().to_owned();
                        let resource = TheMovieDB::get_full_poster_url(&poster, Width::ORIGINAL);
                        self.movie_db.download_poster(&resource, &poster[1..]);
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
            //       it every single frame. We should also not take the slice here because we can
            //       panic since strings are UTF-8 and this takes bytes.
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

            if let Some((search, productions)) = self.fetch_productions_job.poll_owned() {
                let productions: Rc<[Production]> = productions.into();
                self.search_cache.insert(search.clone(), productions.clone());
                self.search_productions = Some(productions);
            }

            let Some(productions) = self.search_productions.clone() else {
                return;
            };

            for prod in &*productions {
                match prod {
                    Production::Movie(ref movie) => self.draw_movie_entry(ui, movie),
                    Production::Series(ref series) => self.draw_series_entry(ui, series),
                }
            }
        });
    }
}
