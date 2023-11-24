use crate::config::Config;
use crate::jobs::Job;
use crate::production::{EntryType, ListEntry, Movie, Production, Series, UserMovie, UserSeries, CentralListOrdering};
use crate::series_details::{SeasonDetails, SeriesDetails};
use crate::themoviedb::{TheMovieDB, Width};
use crate::view::{LicenseView, MovieView, SeriesView, TrailersView};

use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::time::Duration;

use crate::limiter::RateLimiter;
use crate::production;
use egui::ahash::HashSet;
use egui::{Align, Layout, Rect, Response, TopBottomPanel, Ui, Vec2, Visuals, ScrollArea, include_image, RichText};

pub struct MovieApp {
    // Left panel
    search: String,
    show_adult_content: bool,

    search_productions: Option<Rc<[Production]>>,
    search_cache: HashMap<String, Rc<[Production]>>,
    description_cache: HashMap<u32, String>,
    rendered_ids: HashSet<u32>,
    fetch_productions_job: Job<(String, Vec<Production>)>,

    // Central panel
    selected_entry: EntryType,
    // This list holds entries in custom order of the user. Used as a reference for sorting and searching. 
    // It is "mostly" immutable
    central_user_list: Vec<ListEntry>,
    // This list is highly mutable. It will be user for appropriate sorting (when a corresponding ordering 
    // buttons are clicked), shrinking and expanding (when used inputs name of the production in a central panel 
    // search bar). This is the list that is used for central panel drawing.
    central_draw_list: Vec<ListEntry>,
    central_ordering: CentralListOrdering,

    // TODO: Add searchbar and fuzzy searching logic at some point.
    searched_string: String,

    // Right panel
    user_movies: Vec<UserMovie>,
    user_series: Vec<UserSeries>,
    // Notes
    series_details_job: Job<SeriesDetails>,
    season_details_job: Job<SeasonDetails>,
    series_details: Option<SeriesDetails>,
    season_details: Option<SeasonDetails>,
    selection: Selection,

    // View states
    series_view: SeriesView,
    movie_view: MovieView,
    trailers_view: TrailersView,
    license_view: LicenseView,

    // Rate limiter
    image_limiter: RateLimiter,
    // Not a part of the layout
    movie_db: TheMovieDB,
    pub config: Config,
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
            description_cache: HashMap::new(),
            rendered_ids: HashSet::default(),
            fetch_productions_job: Job::Empty,

            user_movies: Vec::new(),
            user_series: Vec::new(),

            series_details_job: Job::Empty,
            season_details_job: Job::Empty,
            series_details: None,
            season_details: None,
            selection: Selection::new(),

            selected_entry: EntryType::None,
            central_user_list: Vec::new(),
            central_draw_list: Vec::new(),
            central_ordering: CentralListOrdering::UserDefined,
            searched_string: String::new(),

            series_view: SeriesView::new(),
            movie_view: MovieView::new(),
            trailers_view: TrailersView::new(),
            license_view: LicenseView::new(),

            image_limiter: RateLimiter::new(20, Duration::from_secs(1)),
            movie_db,
            config,
        }
    }

    fn central_list_reload(&mut self) {
        self.central_user_list.clear();

        // This is not that great. When loading data, custom list order created by the user gets disregarded.
        // Maybe the order of the list should also be saved somehow?

        for series in &self.user_series {
            let entry = ListEntry::from_series(&series.series);
            self.central_user_list.push(entry);
        }

        for movie in &self.user_movies {
            let entry = ListEntry::from_movie(&movie.movie);
            self.central_user_list.push(entry);
        }

        self.central_draw_list_update();
    }

    fn central_list_add_movie(&mut self, movie: &Movie) {
        let entry = ListEntry::from_movie(movie);
        self.central_user_list.push(entry);
        self.central_draw_list_update();
    }

    fn central_list_add_series(&mut self, series: &Series) {
        let entry = ListEntry::from_series(series);
        self.central_user_list.push(entry);
        self.central_draw_list_update();
    }

    fn central_draw_list_update(&mut self) {
        self.central_draw_list.clear();
        // NOTE: I suppose cloning the entries themself is not needed here. Could be improved by storing references,
        //       but of course, this requires a little more work and is more annoying to deal with.
        self.central_draw_list = self.central_user_list.clone();

        match self.central_ordering {
            CentralListOrdering::UserDefined => {},
            CentralListOrdering::Alphabetic => 
                self.central_draw_list.sort_by(|a, b| a.name.cmp(&b.name)),
            CentralListOrdering::RatingAscending => 
                self.central_draw_list.sort_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap()),
            CentralListOrdering::RatingDescending => 
                self.central_draw_list.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap()),
        }

        // NOTE: Definitely needs some improvements, but will do for now.
        //       Also, fuzzy searching would be really nice!
        let matches = self.central_draw_list.iter().filter(|entry|  {
            let searched_lower = self.searched_string.to_lowercase();
            entry.name.to_lowercase().contains(&searched_lower)
        });
        let mut new_draw_list = Vec::new();
        for entry in matches {
            new_draw_list.push(entry.clone());
        }
        self.central_draw_list = new_draw_list;
    }

    pub fn save_data(&self) {
        let outcome = production::serialize_user_productions(&self.user_series, &self.user_movies);
        if outcome.is_err() {
            eprintln!("{}", outcome.unwrap_err())
        }
    }

    pub fn load_data(&mut self) {
        let outcome = production::deserialize_user_productions(None);
        match outcome {
            Ok(user_prods) => {
                self.user_series = user_prods.0;
                self.user_movies = user_prods.1;
            }
            Err(msg) => eprintln!("{}", msg),
        }

        self.central_list_reload();
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

        if self.config.load_on_startup {
            self.load_data();
        }

        self.central_list_reload();
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.series_view.draw(ctx, &self.movie_db);
        self.movie_view.draw(ctx, &self.movie_db);
        self.trailers_view.draw(ctx);
        self.license_view.draw(ctx);

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
            ui.horizontal(|ui| {
                ui.heading("Your movies!");

                ui.with_layout(egui::Layout::right_to_left(Align::Max), |ui| {
                    if ui.button("U").on_hover_text("User defined ordering").clicked() {
                        self.central_ordering = CentralListOrdering::UserDefined;
                        self.central_draw_list_update();
                    }

                    if ui.button("A").on_hover_text("Alphabetic ordering").clicked() {
                        self.central_ordering = CentralListOrdering::Alphabetic;
                        self.central_draw_list_update();
                    }

                    if ui.button("^").on_hover_text("Ascending rating ordering").clicked() {
                        self.central_ordering = CentralListOrdering::RatingAscending;
                        self.central_draw_list_update();
                    }

                    if ui.button("v").on_hover_text("Desceding rating ordering").clicked() {
                        self.central_ordering = CentralListOrdering::RatingDescending;
                        self.central_draw_list_update();
                    }
                });

            });

            ui.vertical_centered_justified(|ui| {
                let search_field = egui::TextEdit::singleline(&mut self.searched_string)
                    .min_size(Vec2::new(20.0, 0.0))
                    .hint_text("Find your movie / series");

                let response = ui.add(search_field);
                if response.changed() {
                    self.central_draw_list_update();
                }
            });

            ui.separator();

            // NOTE: This is an outline of a new list view. Kind of messy and still needs some work.
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for entry in &self.central_draw_list {
                    let desired_size = egui::vec2(ui.available_width(), 32.0);
                    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

                    let mut selected = entry.is_selected(&self.selected_entry);

                    if response.clicked() {
                        if selected {
                            self.selected_entry = EntryType::None;
                            self.selection.index = None;
                        } else {
                            match entry.production_id {
                                EntryType::Movie(id) => {
                                    for (i, movie) in self.user_movies.iter().enumerate() {
                                        if movie.movie.id == id {
                                            self.selection.index = Some(i);
                                        }
                                    }
                                }

                                EntryType::Series(id) => {
                                    for (i, series) in self.user_series.iter().enumerate() {
                                        if series.series.id == id {
                                            self.selection.index = Some(i);
                                            self.selection.season = None;
                                            self.selection.episode = None;
                                            // TODO: Shouldn't be called here. There is no need to call this every
                                            //       time we click on any other series entries
                                            self.series_details_job = self.movie_db.get_series_details(id);
                                        }
                                    }
                                }

                                EntryType::None => unreachable!(),
                            }

                            self.selected_entry = entry.production_id;
                        }

                        selected = !selected;
                    }

                    // Attach some meta-data to the response which can be used by screen readers:
                    // response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, "Something"));

                    if ui.is_rect_visible(rect) {
                        let visuals = ui.style().interact(&response);
                        let visuals2 = ui.style().noninteractive();

                        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
                        let rect = rect.expand(visuals.expansion);

                        if selected {
                            ui.painter().rect(rect, 1.0, visuals.bg_fill, visuals.bg_stroke);
                        } else {
                            ui.painter().rect(rect, 1.0, visuals2.weak_bg_fill, visuals.bg_stroke);
                        }

                        let image_pos = rect.min + egui::vec2(3.0, 3.0);
                        let desired_size = egui::vec2(20.0, 28.0);

                        let image_rect = egui::Rect::from_min_size(image_pos, desired_size);

                        let image = if let Some(ref path) = entry.poster_path {
                            let image_url = TheMovieDB::get_full_poster_url(path, Width::W300);
                            egui::Image::new(image_url)
                        } else {
                            let image_source = include_image!("../res/no_image.png");
                            // let image_source = include_image!("../res/image_unavailable.svg");
                            egui::Image::new(image_source)
                        };

                        image.paint_at(ui, image_rect);

                        let font_pos = rect.min + Vec2::new(32.0, rect.height() / 2.0);
                        let font_id = egui::FontId::new(12.0, eframe::epaint::FontFamily::Proportional);
                        ui.painter().text(
                            font_pos,
                            egui::Align2::LEFT_CENTER,
                            &entry.name,
                            font_id,
                            egui::Color32::GRAY,
                        );

                        // TODO: Add "move up", "move down" and "delete" button that show up
                        //       whenever the list entry has mouse hover
                        //       
                        // TODO: List entries could also be draggable?
                    }
                }

                //
                // NOTE: This shouldn't be needed here anyways. All posters should be cached from the moment when
                //       user adds a new production to the list.
                //
                // if movie.poster_path.is_some() {
                //     let image_url =
                //         TheMovieDB::get_full_poster_url(movie.poster_path.as_ref().unwrap(), Width::W300);
                //
                //     if self.rendered_ids.contains(&movie.id) {
                //         ui.image(image_url);
                //     } else if self.image_limiter.hit() {
                //         self.rendered_ids.insert(movie.id);
                //         ui.image(image_url);
                //     } else {
                //         ui.spinner();
                //     }
                //
                //     ui.heading(&movie.title);
                // }
                //
                // if series.poster_path.is_some() {
                //     let image_url =
                //         TheMovieDB::get_full_poster_url(series.poster_path.as_ref().unwrap(), Width::W300);
                //
                //     if self.rendered_ids.contains(&series.id) {
                //         ui.image(image_url);
                //     } else if self.image_limiter.hit() {
                //         self.rendered_ids.insert(series.id);
                //         ui.image(image_url);
                //     } else {
                //         ui.spinner();
                //     }
                //     ui.heading(&series.name);
                // }
            });
        });
    }

    fn paint_entry(
        &self,
        ui: &mut Ui,
        selected: bool,
        name: &str,
        poster_path: &Option<String>,
        rect: &Rect,
        response: &Response,
    ) {
        if ui.is_rect_visible(*rect) {
            let visuals = ui.style().interact(&response);
            let visuals2 = ui.style().noninteractive();

            // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
            let rect = rect.expand(visuals.expansion);

            if selected {
                ui.painter().rect(rect, 1.0, visuals.bg_fill, visuals.bg_stroke);
            } else {
                ui.painter().rect(rect, 1.0, visuals2.weak_bg_fill, visuals.bg_stroke);
            }

            let pos = rect.min + Vec2::new(32.0, rect.height() / 2.0);
            let font_id = egui::FontId::new(12.0, eframe::epaint::FontFamily::Proportional);
            ui.painter()
                .text(pos, egui::Align2::LEFT_CENTER, name, font_id, egui::Color32::GRAY);

            if let Some(ref path) = poster_path {
                let image_pos = rect.min + egui::vec2(3.0, 3.0);
                let desired_size = egui::vec2(20.0, 28.0);

                let image_rect = egui::Rect::from_min_size(image_pos, desired_size);
                let image_url = TheMovieDB::get_full_poster_url(path, Width::W300);
                egui::Image::new(image_url).paint_at(ui, image_rect);
            }
        }
    }

    fn right_panel(&mut self, ctx: &egui::Context) {
        let right = egui::SidePanel::right("right_panel");
        // This needs a lot of changes
        right.resizable(true).show(ctx, |ui| {
            let heading;
            let is_movie;
            match self.selected_entry {
                EntryType::Movie(_) => {
                    heading = "Selected movie";
                    is_movie = true;
                }
                EntryType::Series(_) => {
                    // assuming we reset selected_user_movie
                    heading = "Selected series";
                    is_movie = false;
                }
                EntryType::None => {
                    ui.heading("Nothing selected");
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label("Currently nothing is selected ._.");
                    return;
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
            let index = self.selection.index();
            if is_movie {
                let Some(user_movie) = self.user_movies.get_mut(index) else {
                    return;
                };
                let movie = &user_movie.movie;
                ui.heading(&movie.title);

                if let Some(poster) = &movie.poster_path {
                    let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                    let image = egui::Image::new(image_url);
                    ui.add_sized([100.0, 100.0], image);
                }
            } else {
                let Some(user_series) = self.user_series.get_mut(index) else {
                    return;
                };
                let series = &user_series.series;
                ui.heading(&series.name);

                if let Some(poster) = &series.poster_path {
                    let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                    let image = egui::Image::new(image_url);
                    ui.add_sized([100.0, 100.0], image);
                }

                if let Some(details) = self.series_details_job.poll_owned() {
                    self.series_details = Some(details);
                }
                if let Some(details) = &self.series_details {
                    let display = if self.selection.season.is_some() {
                        format!("S{}", self.selection.season())
                    } else {
                        "None".to_string()
                    };
                    let before_render_season = self.selection.season.unwrap_or(0);
                    egui::ComboBox::from_label("Select season!")
                        .selected_text(display)
                        .show_ui(ui, |ui| {
                            for i in 1..=details.number_of_seasons {
                                ui.selectable_value(&mut self.selection.season, Some(i), format!("S{}", i));
                            }
                            ui.selectable_value(&mut self.selection.season, None, "None");
                        });

                    let after_render_season = self.selection.season.unwrap_or(0);
                    if before_render_season != after_render_season {
                        self.selection.episode = None;
                    }
                    if let Some(season_num) = self.selection.season {
                        let season_num = season_num as usize;
                        let display = if let Some(episode) = self.selection.episode {
                            format!("EP{}", episode)
                        } else {
                            "None".to_string()
                        };
                        let all_episodes;
                        if details.has_specials() {
                            all_episodes = details.seasons[season_num].episode_count;
                        } else {
                            all_episodes = details.seasons[season_num - 1].episode_count;
                        }
                        egui::ComboBox::from_label("Select episode!")
                            .selected_text(display)
                            .show_ui(ui, |ui| {
                                for i in 1..=all_episodes {
                                    ui.selectable_value(&mut self.selection.episode, Some(i), format!("EP{}", i));
                                }
                                ui.selectable_value(&mut self.selection.episode, None, "None");
                            });
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
                user_movie = self.user_movies.get_mut(self.selection.index()).unwrap();
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
                user_series = self.user_series.get_mut(self.selection.index()).unwrap();
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
                if let Some(episode_num) = self.selection.episode {
                    let season_num = self.selection.season();
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
                if let Some(season_num) = self.selection.season {
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
                    if ui.button("Save data").clicked() {
                        self.save_data();
                    }

                    if ui.button("Load data").clicked() {
                        self.load_data();
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

                    if ui.button("Auto-save").clicked() {
                        todo!()
                    }

                    if ui.button("Enable caching").clicked() {
                        todo!()
                    }

                    if ui.button("Set TMDB token").clicked() {
                        todo!()
                    }

                    if ui.button("Set default browser").clicked() {
                        todo!()
                    }

                    if ui.button("Sync").clicked() {
                        todo!()
                    }
                });

                ui.menu_button("About", |_| {});
                ui.menu_button("License", |ui| {
                    self.license_view.is_open = true;
                });
            });
        });
    }

    fn draw_movie_entry(&mut self, ui: &mut Ui, movie: &Movie) {
        if movie.adult && !self.show_adult_content {
            return;
        }

        ui.horizontal(|ui| {
            let image = if let Some(poster) = &movie.poster_path {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                egui::Image::new(image_url)
            } else {
                egui::Image::new(include_image!("../res/no_image.png"))
                // egui::Image::new(include_image!("../res/image_unavailable.svg"))
            };

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
                        self.central_list_add_movie(movie);
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
                    let url = self.movie_db.get_imdb_url_movie(&movie.title, movie.id);
                    let browser = &self.config.browser_name;
                    let _ = open::with_in_background(url, browser);
                }

                if ui.button("Fetch keywords").clicked() {
                    let keywords = self.movie_db.get_keywords_movie(movie.id);
                    println!("{:?}", keywords)
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
            // NOTE: It's not that bad now!
            if self.description_cache.contains_key(&movie.id) {
                let description = self.description_cache.get(&movie.id).expect("Not cached");
                ui.label(description);
            } else {
                let slice = &movie.overview.as_bytes()[..200];
                let description = format!("{}...", String::from_utf8_lossy(slice).trim());
                ui.label(&description);
                self.description_cache.insert(movie.id, description);
            }
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
            let image = if let Some(poster) = &series.poster_path {
                let image_url = TheMovieDB::get_full_poster_url(poster, Width::W300);
                egui::Image::new(image_url)
            } else {
                egui::Image::new(include_image!("../res/no_image.png"))
                // egui::Image::new(include_image!("../res/image_unavailable.svg"))
            };

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
                        self.central_list_add_series(series);
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
                    let url = self.movie_db.get_imdb_url_series(&series.name, series.id);
                    let browser = &self.config.browser_name;
                    let _ = open::with_in_background(url, browser);
                }

                if ui.button("Fetch trailers").clicked() {
                    let trailers = self.movie_db.get_series_trailers(series.id);
                    self.trailers_view.set_content(series.name.to_owned(), trailers);
                }

                if ui.button("Fetch keywords").clicked() {
                    let keywords = self.movie_db.get_keywords_series(series.id);
                    println!("{:?}", keywords)
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
            // NOTE: It's not that bad now!
            if self.description_cache.contains_key(&(series.id + 1)) {
                let description = self.description_cache.get(&(series.id + 1)).expect("Not cached");
                ui.label(description);
            } else {
                let slice = &series.overview.as_bytes()[..200];
                let description = format!("{}...", String::from_utf8_lossy(slice).trim());
                ui.label(&description);
                self.description_cache.insert(series.id + 1, description);
            }
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

struct Selection {
    //index into user movies / user series, depending on selected_entry
    index: Option<usize>,
    season: Option<u32>,  //cannot be 0
    episode: Option<u32>  //cannot be 0
}
impl Selection {
    pub fn new() -> Self{
        Self{
            index: None,
            season: None,
            episode: None
        }
    }
    pub fn unselect_all(&mut self) {
        self.index = None;
        self.season = None;
        self.episode = None;
    }
    pub fn index(&self) -> usize {
        self.index.expect("Selection index is None")
    }
    pub fn season(&self) -> u32 {
        self.season.expect("Selection season is None")
    }
    pub fn episode(&self) -> u32 {
        self.episode.expect("Selection episode is None")
    }
}
