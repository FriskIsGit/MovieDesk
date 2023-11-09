use crate::{production::{Movie, Series, Trailer}, jobs::Job, series_details::{SeriesDetails, SeasonDetails}, themoviedb::{TheMovieDB, Width}};

use egui::{ImageSource::Uri, Sense, include_image, Label};

pub struct ExpandedView {
    series_window_open: bool,
    movie_window_open: bool,
    series_window_title: String,
    _movie_window_title: String,
    series: Option<Series>,
    movie: Option<Movie>,

    series_details: Job<SeriesDetails>,
    season_details: Job<SeasonDetails>,

    expanded_season: bool,
}

pub struct TrailersView {
    is_open: bool,
    title: String,
    trailers: Vec<Trailer>,
}

impl ExpandedView {
    pub fn new() -> Self {
        Self {
            series_window_open: false,
            movie_window_open: false,
            series_window_title: "".into(),
            _movie_window_title: "".into(),
            series: None,
            movie: None,

            series_details: Job::Empty,
            season_details: Job::Empty,

            expanded_season: false,
        }
    }

    pub fn set_movie(&mut self, _movie: Movie) {
        /*let id = movie.id;
        let movie_window_title = movie.name.clone();
        self.movie = Some(movie);
        let movie_db = self.movie_db.clone();
        let handle = thread::spawn(move || {
            Some(movie_db.get_movie_details(id))
        });
        self.movie_details_job.set(handle);*/
    }

    pub fn set_series(&mut self, series: Series, movie_db: &TheMovieDB) {
        let id = series.id;
        self.series_window_title = series.name.clone();
        self.series = Some(series);

        self.series_details = movie_db.get_series_details(id);
        self.season_details = Job::Empty;
        self.series_window_open = true;
    }

    //this is called every frame
    pub fn expanded_series_window(&mut self, ctx: &egui::Context, movie_db: &TheMovieDB) {
        let Some(series) = self.series.as_ref() else { return };
        let Some(series_details) = self.series_details.poll() else {
            return;
        };

        // self.series_window_open = true;

        let seasons_per_row = std::cmp::min(5, series_details.seasons.len());

        let window = egui::Window::new(&self.series_window_title)
            .open(&mut self.series_window_open)
            .default_width((seasons_per_row * 100 + seasons_per_row * 5) as f32)
            .default_height(300.0)
            .resizable(true);

        window.show(ctx, |ui| {
            if self.expanded_season {
                let Some(season_details) = self.season_details.poll() else {
                    return;
                };

                if ui.button("<=").clicked() {
                    self.expanded_season = false;
                    self.series_window_title = series.name.clone();
                    return;
                }

                ui.label(format!("Watch time: {}", season_details.runtime()));
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

            egui::ScrollArea::new([true; 2]).show(ui, |ui| {
                egui::Grid::new("expanded_view").max_col_width(100.0).show(ui, |ui| {
                    for (i, season) in series_details.seasons.iter().enumerate() {
                        ui.vertical(|ui| {
                            // it's a bad idea to fetch posters for every season
                            let image = match season.poster_path.as_ref() {
                                Some(url) => {
                                    let image_url = TheMovieDB::get_full_poster_url(url, Width::W300);
                                    egui::Image::new(Uri(image_url.into())).sense(Sense::click())
                                }
                                None => egui::Image::new(include_image!("../res/no_image.png")).sense(Sense::click()),
                            };

                            let poster_response = ui.add_sized([100.0, (100.0 / 60.0) * 100.0], image);
                            let label_response = ui.add(Label::new(&season.name).sense(Sense::click()));

                            if poster_response.clicked() || label_response.clicked() {
                                self.expanded_season = true;
                                self.series_window_title = format!("{} -> {}", self.series_window_title, season.name);

                                let series_id = series.id;
                                let season_number = season.season_number;

                                self.season_details = movie_db.get_season_details(series_id, season_number);
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

    pub fn expanded_movie_window(&mut self, ctx: &egui::Context) {
        let Some(_) = self.movie.as_ref() else {
            return;
        };

        println!("Expanded");
        let movie = &self.movie.as_ref().unwrap();

        let window = egui::Window::new(&movie.title)
            .open(&mut self.movie_window_open)
            .resizable(true);

        window.show(ctx, |ui| ui.label("Hello movie!"));
    }
}

impl TrailersView {
    pub fn new() -> Self {
        Self {
            is_open: false,
            title: "".into(),
            trailers: Vec::new(),
        }
    }

    pub fn set_content(&mut self, title: String, trailers: Vec<Trailer>) {
        self.is_open = true;
        self.title = title;
        self.trailers = trailers;
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        if !self.is_open || self.trailers.is_empty() {
            return;
        }
        let window = egui::Window::new(&self.title).open(&mut self.is_open).resizable(true);

        window.show(ctx, |ui| {
            ui.vertical(|ui| {
                for trailer in &self.trailers {
                    ui.label(&trailer.name);
                    ui.hyperlink(trailer.youtube_url());
                    ui.separator();
                }
            });
        });
    }
}
