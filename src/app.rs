use std::cmp::min;
use crate::config::Config;
use crate::jobs::Job;
use crate::production::{Movie, Production, Series, UserProduction};
use crate::series_details::{SeasonDetails, SeriesDetails};
use crate::themoviedb::{TheMovieDB, Width};

use egui;
use egui::ImageSource::Uri;
use egui::{Align, Align2, include_image, Label, Layout, Sense, TopBottomPanel, Ui, Vec2, Visuals};

use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::thread;

pub struct MovieApp {
    // Left panel
    search: String,
    show_adult_content: bool,
    search_productions: Vec<Production>,

    // Right and center panel
    user_productions: Vec<UserProduction>,
    selected_user_production: Option<usize>,

    // Expanded view state
    expanded_view: ExpandedView,
    // Jobs
    fetch_productions_job: Job<Vec<Production>>,

    // Not a part of the layout
    movie_db: Arc<TheMovieDB>,
    config: Config,
}

impl MovieApp {
    pub fn new(ctx: &egui::Context, mut config: Config) -> Self {
        let visuals = Visuals::dark();
        ctx.set_visuals(visuals);

        // Implement dynamic scale changing?
        ctx.set_pixels_per_point(1.66);

        // YOINK! We are not going to these anymore.
        let key = std::mem::take(&mut config.api_key);
        let cache = std::mem::take(&mut config.enable_cache);

        let movie_db = Arc::new(TheMovieDB::new(key, cache));
        Self {
            search: String::new(),
            show_adult_content: config.include_adult,
            search_productions: Vec::new(),

            user_productions: Vec::new(),
            selected_user_production: None,

            expanded_view: ExpandedView::new(movie_db.clone()),
            fetch_productions_job: Job::empty(),

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
        self.expanded_view.expanded_series_window(ctx);
        //self.expanded_view.expanded_movie_window(ctx, self.movie_db);

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
                    let movie_db = self.movie_db.clone();
                    let search = self.search.to_owned();
                    let handle = thread::spawn(move || movie_db.search_production(&search));
                    self.fetch_productions_job.set(handle);
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
                    let entries = &self.user_productions;
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
                                        &movie.poster_path.as_ref().unwrap(),
                                        Width::W300,
                                    );

                                    ui.image(Uri(image_url.into()));
                                    ui.heading(&movie.title);
                                }
                            }
                            Production::Series(series) => {
                                if series.poster_path.is_some() {
                                    let image_url = TheMovieDB::get_full_poster_url(
                                        &series.poster_path.as_ref().unwrap(),
                                        Width::W300,
                                    );

                                    ui.image(Uri(image_url.into()));
                                    ui.heading(&series.name);
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }

    fn right_panel(&mut self, ctx: &egui::Context) {
        let right = egui::SidePanel::right("right_panel");
        right.show(ctx, |ui| {
            ui.heading("Selected production");
            ui.separator();

            let Some(index) = self.selected_user_production else {
                ui.add_space(10.0);
                ui.label("Currently nothing is selected ._.");
                return;
            };

            // let mut user_productions = self.user_productions.borrow_mut();
            let Some(entry) = self.user_productions.get_mut(index) else {
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

    fn draw_movie_entry(&mut self, ui: &mut Ui, movie: &Movie) {
        if movie.adult && !self.show_adult_content {
            return;
        }

        ui.horizontal(|ui| {
            if let Some(poster) = &movie.poster_path {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);

                let image = egui::Image::new(Uri(image_url.into()));
                let poster = ui.add_sized([60.0, 100.0], image).interact(egui::Sense::click());
                poster.context_menu(|ui| {
                    if ui.button("Add movie").clicked() {
                        // let mut user_productions = self.user_productions.borrow_mut();
                        let exists = self.user_productions.iter().any(|entry| {
                            let Production::Movie(user_movie) = &entry.production else {
                                return false;
                            };
                            user_movie.id == movie.id
                        });

                        if !exists {
                            let new_data = UserProduction {
                                production: Production::Movie(movie.clone()),
                                user_note: String::new(),
                                user_rating: 0.0,
                            };
                            self.user_productions.push(new_data);
                        }
                        ui.close_menu()
                    }
                    //change name?: xpanded view, about, more, view seasons, view more, view details,
                    if ui.button("More details").clicked() {
                        self.expanded_view.set_movie(movie.clone());
                        ui.close_menu();
                    }

                    if ui.button("Open in tmdb").clicked() {
                        let mut path = String::from(MOVIE_URL);
                        path.push_str(movie.id.to_string().as_str());
                        let browser = &self.config.browser_name;
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Open in imdb").clicked() {
                        let path = format!("https://www.imdb.com/find/?q={}", movie.title);
                        let browser = &self.config.browser_name;
                        //use External IDs (movie endpoint)
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Download poster").clicked() && movie.poster_path.is_some() {
                        let poster = movie.poster_path.clone().unwrap().to_owned();
                        let resource = TheMovieDB::get_full_poster_url(&poster, Width::ORIGINAL);
                        let movie_db = self.movie_db.clone();
                        thread::spawn(move || {
                            let bytes = movie_db.download_resource(resource.as_str());
                            let mut file = File::create(&poster[1..]).expect("Unable to create file");
                            // Write a slice of bytes to the file
                            file.write_all(&bytes).unwrap();
                        });
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
                        let exists = self.user_productions.iter().any(|entry| {
                            let Production::Series(user_series) = &entry.production else {
                                return false;
                            };
                            user_series.id == series.id
                        });

                        if !exists {
                            let new_data = UserProduction {
                                production: Production::Series(series.clone()),
                                user_note: String::new(),
                                user_rating: 0.0,
                            };
                            self.user_productions.push(new_data);
                        }
                        ui.close_menu()
                    }

                    if ui.button("More series details").clicked() {
                        self.expanded_view.set_series(series.clone());
                        ui.close_menu();
                    }

                    if ui.button("Open in tmdb").clicked() {
                        let mut path = String::from(TV_URL);
                        path.push_str(series.id.to_string().as_str());
                        let browser = &self.config.browser_name;
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Open in imdb").clicked() {
                        let path = format!("https://www.imdb.com/find/?q={}", series.name);
                        let browser = &self.config.browser_name;
                        //its buggy open in tmdb instead?
                        let _ = open::with_in_background(path, browser);
                    }

                    if ui.button("Download poster").clicked() && series.poster_path.is_some() {
                        let poster = series.poster_path.clone().unwrap().to_owned();
                        let resource = TheMovieDB::get_full_poster_url(&poster, Width::ORIGINAL);
                        let movie_db = self.movie_db.clone();
                        thread::spawn(move || {
                            let bytes = movie_db.download_resource(resource.as_str());
                            let mut file = File::create(&poster[1..]).expect("Unable to create file");
                            // Write a slice of bytes to the file
                            file.write_all(&bytes).unwrap();
                        });
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

            if self.fetch_productions_job.is_any_and_finished() {
                self.search_productions = self.fetch_productions_job.take_result().expect("No productions")
            }
            let mut productions = std::mem::take(&mut self.search_productions);
            for prod in productions.iter() {
                match prod {
                    Production::Movie(movie) => self.draw_movie_entry(ui, movie),
                    Production::Series(series) => self.draw_series_entry(ui, series),
                }
            }
            std::mem::swap(&mut productions, &mut self.search_productions);
        });
    }
}

struct ExpandedView {
    series_window_open: bool,
    movie_window_open: bool,
    series_window_title: String,
    movie_window_title: String,
    series: Option<Series>,
    movie: Option<Movie>,

    series_details: Option<SeriesDetails>,
    series_details_job: Job<Option<SeriesDetails>>,

    season_details: Option<SeasonDetails>,
    season_details_job: Job<Option<SeasonDetails>>,
    expanded_season: bool,

    movie_db: Arc<TheMovieDB>,
}

impl ExpandedView {
    pub fn new(movie_db: Arc<TheMovieDB>) -> Self {
        Self {
            series_window_open: false,
            movie_window_open: false,
            series_window_title: "".into(),
            movie_window_title: "".into(),
            series: None,
            movie: None,

            series_details: None,
            series_details_job: Job::empty(),

            season_details: None,
            season_details_job: Job::empty(),
            expanded_season: false,

            movie_db,
        }
    }

    fn set_movie(&mut self, movie: Movie) {
        /*let id = movie.id;
        let movie_window_title = movie.name.clone();
        self.movie = Some(movie);
        let movie_db = self.movie_db.clone();
        let handle = thread::spawn(move || {
            Some(movie_db.get_movie_details(id))
        });
        self.movie_details_job.set(handle);*/
    }

    fn set_series(&mut self, series: Series) {
        let id = series.id;
        self.series_window_title = series.name.clone();
        self.series = Some(series);
        let movie_db = self.movie_db.clone();
        let handle = thread::spawn(move || Some(movie_db.get_series_details(id)));
        self.season_details = None;
        self.series_details_job.set(handle);
    }

    //this is called every frame
    fn expanded_series_window(&mut self, ctx: &egui::Context) {
        if self.series.is_none() {
            return;
        }
        if self.series_details_job.is_any_and_finished() {
            self.series_details = self.series_details_job.take_result().expect("No series details :(");
            self.series_window_open = true;
        }
        if self.series_details.is_none() {
            return;
        }
        //Displayable state reached
        let series = &self.series.as_ref().unwrap();
        let series_details = &self.series_details.as_ref().unwrap();
        let seasons_per_row = min(5, series_details.seasons.len());
        let window = egui::Window::new(&self.series_window_title)
            .open(&mut self.series_window_open)
            .default_width((seasons_per_row * 100 + seasons_per_row * 5) as f32)
            .default_height(300.0)
            .resizable(true);
        window.show(ctx, |ui| {
            if self.expanded_season && self.season_details.is_some() {
                if ui.button("<=").clicked() {
                    self.expanded_season = false;
                    self.series_window_title = series.name.clone();
                    return;
                }
                let season_details = self.season_details.as_ref().expect("Season details was None");
                ui.label(format!("Watch time: {}min = {}hr", season_details.runtime_minutes(), season_details.runtime_hours()));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for episode in &season_details.episodes {
                                ui.label(format!("{}# {}", episode.episode_number, episode.name));
                            }
                        });
                    });
                });
                return;
            }
            ui.label(&series.overview);
            ui.label(format!("Seasons: {}", series_details.number_of_seasons));
            ui.label(format!("Episodes: {}", series_details.number_of_episodes));
            ui.label(format!("Status: {}", series_details.status));
            ui.separator();
            egui::ScrollArea::new([true, true]).show(ui, |ui| {
                egui::Grid::new("expanded_view").max_col_width(100.0).show(ui, |ui| {
                    for (i, season) in series_details.seasons.iter().enumerate() {
                        ui.vertical(|ui| {
                            //it's a bad idea to fetch posters for every season
                            let mut image;
                            if season.poster_path.is_some() {
                                let image_url =
                                    TheMovieDB::get_full_poster_url(&season.poster_path.as_ref().unwrap(), Width::W300);
                                image = egui::Image::new(Uri(image_url.into())).sense(Sense::click());
                            } else {
                                image = egui::Image::new(include_image!("../res/no_image.png")).sense(Sense::click());
                            }
                            let poster_response = ui.add_sized([100.0, (100.0 / 60.0) * 100.0], image);
                            if poster_response.clicked() {
                                self.expanded_season = true;
                                self.series_window_title = format!("{} -> {}", self.series_window_title, season.name.clone());
                                self.season_details =
                                    Some(self.movie_db.get_season_details(series.id, season.season_number));
                            }
                            let label_response = ui.add(Label::new(&season.name).sense(Sense::click()));
                            if label_response.clicked() {
                                self.expanded_season = true;
                                self.series_window_title = format!("{} -> {}", self.series_window_title, season.name.clone());
                                self.season_details =
                                    Some(self.movie_db.get_season_details(series.id, season.season_number));
                            }
                        });

                        if (i + 1) % 5 == 0 {
                            ui.end_row()
                        }
                    }
                });
            });
        });
    }

    fn expanded_movie_window(&mut self, ctx: &egui::Context) {
        println!("Expanded");
        let movie = &self.movie.as_ref().unwrap();
        //Expanded
        let window = egui::Window::new(&movie.title)
            .open(&mut self.movie_window_open)
            .resizable(true);
        window.show(ctx, |ui| {
            ui.label("Hello movie!");
        });
    }
}
