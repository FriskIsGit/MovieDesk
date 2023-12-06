use crate::config::Config;
use crate::jobs::Job;
use crate::movies::{Movie, UserMovie};
use crate::production::{ListOrdering, EntryType, ListEntry, Production, ProdEntry, ListFiltering};
use crate::series::{SearchedSeries, UserSeries, Series};
use crate::themoviedb::{TheMovieDB, Width};
use crate::view::{LicenseView, MovieView, SeriesView, TrailersView};

use std::collections::{HashMap, hash_map};
use std::ops::RangeInclusive;
use std::rc::Rc;

use crate::production;
use egui::{include_image, Align, Layout, Pos2, Rect, TopBottomPanel, Ui, Vec2, Visuals};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

pub struct MovieApp {
    // Left panel
    search: String,
    show_adult_content: bool,

    search_productions: Option<Rc<[Production]>>,
    fetch_productions_job: Job<Vec<Production>>,

    description_cache: HashMap<u32, String>,

    // Central panel
    selected_entry: EntryType,
    // This list holds entries in custom order of the user. Used as a reference for sorting and searching.
    // It is "mostly" immutable
    central_user_list: Vec<ListEntry>,
    // This list is highly mutable. It will be user for appropriate sorting (when a corresponding ordering
    // buttons are clicked), shrinking and expanding (when used inputs name of the production in a central panel
    // search bar). This is the list that is used for central panel drawing.
    central_draw_list: Vec<ListEntry>,
    central_ordering:  ListOrdering,
    central_filtering: ListFiltering,
    searched_string:   String,

    // Right panel
    user_movies:    Vec<UserMovie>,
    user_series:    Vec<UserSeries>,
    prod_positions: Vec<ProdEntry>,
    selection:      Selection,

    toasts: Toasts,

    // View states
    series_view: SeriesView,
    movie_view: MovieView,
    trailers_view: TrailersView,
    license_view: LicenseView,

    // Not a part of the layout
    movie_db: TheMovieDB,
    pub config: Config,
}

impl MovieApp {
    pub fn new(ctx: &egui::Context, config: Config) -> Self {
        let visuals = Visuals::dark();
        ctx.set_visuals(visuals);

        // Implement dynamic scale changing?
        ctx.set_pixels_per_point(1.5);

        // NOTE: TheMovieDB SHOULDN'T hold the api_key, this struct is dumb!
        //       #AbolishTheMovieDB
        let key = config.access_token.clone();
        let movie_db = TheMovieDB::new(key, config.enable_cache);

        Self {
            search: String::new(),
            show_adult_content: config.include_adult,
            search_productions: None,
            description_cache: HashMap::new(),
            fetch_productions_job: Job::Empty,

            user_movies: Vec::new(),
            user_series: Vec::new(),
            prod_positions: Vec::new(),

            selection: Selection::new(),

            selected_entry: EntryType::None,
            central_user_list: Vec::new(),
            central_draw_list: Vec::new(),
            central_ordering: ListOrdering::UserDefined,
            central_filtering: ListFiltering::new(),
            searched_string: String::new(),

            toasts: Toasts::new()
                .anchor(egui::Align2::RIGHT_TOP, (1.0, 1.0))
                .direction(egui::Direction::TopDown),
            series_view: SeriesView::new(),
            movie_view: MovieView::new(),
            trailers_view: TrailersView::new(),
            license_view: LicenseView::new(),

            movie_db,
            config,
        }
    }

    fn central_list_reload(&mut self) {
        self.central_user_list.clear();

        for i in 0..self.prod_positions.len() {
            let pos = self.prod_positions[i];
            if pos.is_movie {
                for i in 0..self.user_movies.len() {
                    //                                    cringe
                    let movie = self.user_movies[i].clone();
                    if movie.movie.id == pos.id {
                        self.central_list_add_movie(&movie);
                    }
                }
            } else {
                for i in 0..self.user_series.len() {
                    //                               cringe
                    let series = self.user_series[i].clone();
                    if series.series.id == pos.id {
                        self.central_list_add_series(&series);
                    }
                }
            }
        }

        self.central_draw_list_update();
    }

    fn central_list_add_movie(&mut self, movie: &UserMovie) {
        let entry = ListEntry::from_movie(movie);
        self.central_user_list.push(entry);
        self.central_draw_list_update();
    }

    fn central_list_add_series(&mut self, series: &UserSeries) {
        let entry = ListEntry::from_series(series);
        self.central_user_list.push(entry);
        self.central_draw_list_update();
    }

    fn central_user_list_move_down(&mut self, index: usize) {
        if !matches!(self.central_ordering, ListOrdering::UserDefined) {
            return;
        } 

        if !self.searched_string.is_empty() {
            return;
        }

        if index + 1 >= self.central_user_list.len() {
            return;
        }

        self.central_user_list.swap(index, index + 1);
        self.prod_positions.swap(index, index + 1);
        self.central_draw_list_update();
    }

    fn central_user_list_move_up(&mut self, index: usize) {
        if !matches!(self.central_ordering, ListOrdering::UserDefined) {
            return;
        } 

        if !self.searched_string.is_empty() {
            return;
        }

        if index == 0 {
            return;
        }

        self.central_user_list.swap(index, index - 1);
        self.prod_positions.swap(index, index - 1);
        self.central_draw_list_update();
    }

    fn central_draw_list_update(&mut self) {
        self.central_draw_list.clear();

        let searched_lower = self.searched_string.to_lowercase();

        let mut new_draw_list = Vec::new();
        for entry in &self.central_user_list {
            if self.central_filtering.filter_favorites && !entry.favorite {
                continue;
            }

            if self.central_filtering.filter_watched && !entry.watched {
                continue;
            }

            if !entry.name.to_lowercase().contains(&searched_lower) {
                continue;
            }

            new_draw_list.push(entry.clone());
        }

        // NOTE: I suppose cloning the entries themself is not needed here. Could be improved by storing references,
        //       but of course, this requires a little more work and is more annoying to deal with.
        self.central_draw_list = new_draw_list;

        match self.central_ordering {
            ListOrdering::UserDefined => {}
            ListOrdering::Alphabetic => 
                self.central_draw_list.sort_by(|a, b| a.name.cmp(&b.name)),
            ListOrdering::RatingAscending => 
                self.central_draw_list.sort_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap()),
            ListOrdering::RatingDescending => 
                self.central_draw_list .sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap()),
        }

        // // NOTE: Definitely needs some improvements, but will do for now.
        // //       Also, fuzzy searching would be really nice!
        // let matches = self.central_draw_list.iter().filter(|entry| {
        //     let searched_lower = self.searched_string.to_lowercase();
        //     entry.name.to_lowercase().contains(&searched_lower)
        // });
        // let mut new_draw_list = Vec::new();
        // for entry in matches {
        //     new_draw_list.push(entry.clone());
        // }
        // self.central_draw_list = new_draw_list;
    }

    fn central_list_handle_selection(&mut self, entry_id: EntryType, is_selected: bool) {
        if is_selected {
            self.selected_entry = EntryType::None;
            self.selection.index = None;
        } else {
            match entry_id {
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
                        }
                    }
                }

                EntryType::None => unreachable!(),
            }

            self.selected_entry = entry_id;
        }
    }

    // TODO: Improve this
    fn central_list_remove_entry(&mut self, entry_id: EntryType) {
        self.selected_entry = EntryType::None;
        self.selection.index = None;

        match entry_id {
            EntryType::Movie(id) => {
                let mut found_idx = 0;
                for (i, movie) in self.user_movies.iter().enumerate() {
                    if movie.movie.id == id {
                        found_idx = i;
                        break;
                    }
                }
                self.user_movies.remove(found_idx);
            }
            EntryType::Series(id) => {
                let mut found_idx = 0;
                for (i, series) in self.user_series.iter().enumerate() {
                    if series.series.id == id {
                        found_idx = i;
                        break;
                    }
                }
                self.user_series.remove(found_idx);
            }
            EntryType::None => unreachable!(),
        }

        self.central_list_reload();
    }

    fn central_list_mark_watched(&mut self, entry_id: EntryType) {
        match entry_id {
            EntryType::Movie(id) => {
                for movie in self.user_movies.iter_mut() {
                    if movie.movie.id == id {
                        movie.watched = !movie.watched;
                        break;
                    }
                }
            }
            EntryType::Series(id) => {
                for series in self.user_series.iter_mut() {
                    if series.series.id == id {
                        series.watched = !series.watched;
                        break;
                    }
                }
            }
            EntryType::None => unreachable!(),
        }

        self.central_list_reload();
    }

    fn central_list_mark_favorite(&mut self, entry_id: EntryType) {
        match entry_id {
            EntryType::Movie(id) => {
                for movie in self.user_movies.iter_mut() {
                    if movie.movie.id == id {
                        movie.favorite = !movie.favorite;
                        break;
                    }
                }
            }
            EntryType::Series(id) => {
                for series in self.user_series.iter_mut() {
                    if series.series.id == id {
                        series.favorite = !series.favorite;
                        break;
                    }
                }
            }
            EntryType::None => unreachable!(),
        }

        self.central_list_reload();
    }

    pub fn save_data(&mut self) {
        let outcome = production::serialize_user_productions(&self.user_series, &self.user_movies, &self.prod_positions);
        match outcome {
            Ok(_) => {
                self.toasts.add(Toast {
                    text: "Saved productions".into(),
                    kind: ToastKind::Success,
                    options: ToastOptions::default()
                        .duration_in_seconds(2.5)
                        .show_progress(true)
                        .show_icon(true),
                });
            }
            Err(msg) => {
                eprintln!("{}", msg);
                self.toasts.add(Toast {
                    text: msg.into(),
                    kind: ToastKind::Error,
                    options: ToastOptions::default()
                        .duration_in_seconds(3.5)
                        .show_progress(true)
                        .show_icon(true),
                });
            }
        }
    }

    pub fn load_data(&mut self) {
        let outcome = production::deserialize_user_productions(None);
        match outcome {
            Ok(user_data) => {
                self.user_series = user_data.user_series;
                self.user_movies = user_data.user_movies;
                self.prod_positions = user_data.prod_positions;
                self.toasts.add(Toast {
                    text: "Loaded productions".into(),
                    kind: ToastKind::Success,
                    options: ToastOptions::default()
                        .duration_in_seconds(2.5)
                        .show_progress(true)
                        .show_icon(true),
                });
            }
            Err(msg) => {
                eprintln!("{}", msg);
                self.toasts.add(Toast {
                    text: msg.into(),
                    kind: ToastKind::Error,
                    options: ToastOptions::default()
                        .duration_in_seconds(3.5)
                        .show_progress(true)
                        .show_icon(true),
                });
            }
        }

        self.central_list_reload();
    }

    pub fn add_movie(&mut self, movie: Movie) {
        let exists = self.user_movies.iter()
            .any(|user_movie| user_movie.movie.id == movie.id);

        if !exists {
            self.prod_positions.push(ProdEntry::new(true, movie.id));
            let new_data = UserMovie::new(movie.clone());
            self.central_list_add_movie(&new_data);
            self.user_movies.push(new_data);
        }
    }


    pub fn add_series(&mut self, series: Series) {
        let exists = self.user_series.iter()
            .any(|user_series| user_series.series.id == series.id);

        if !exists {
            self.prod_positions.push(ProdEntry::new(false, series.id));
            let new_data = UserSeries::new(series);
            self.central_list_add_series(&new_data);
            self.user_series.push(new_data);
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

        // Show all toasts
        self.toasts.show(ctx);

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
                    self.fetch_productions_job = self.movie_db.search_production(self.search.clone());
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
        // NOTE:
        //     We could also add option exclude or find productions by keyword.
        //     Searching by both production title and keywords could also be interesting.
        let center = egui::CentralPanel::default();
        center.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Your movies!");

                ui.with_layout(egui::Layout::right_to_left(Align::Max), |ui| {
                    if ui.button("U").on_hover_text("User defined ordering").clicked() {
                        self.central_ordering = ListOrdering::UserDefined;
                        self.central_draw_list_update();
                    }

                    if ui.button("A").on_hover_text("Alphabetic ordering").clicked() {
                        self.central_ordering = ListOrdering::Alphabetic;
                        self.central_draw_list_update();
                    }

                    if ui.button("^").on_hover_text("Ascending rating ordering").clicked() {
                        self.central_ordering = ListOrdering::RatingAscending;
                        self.central_draw_list_update();
                    }

                    if ui.button("v").on_hover_text("Descending rating ordering").clicked() {
                        self.central_ordering = ListOrdering::RatingDescending;
                        self.central_draw_list_update();
                    }

                    if ui.button("F").on_hover_text("Descending rating ordering").clicked() {
                        self.central_filtering.filter_favorites = !self.central_filtering.filter_favorites;
                        self.central_draw_list_update();
                    }

                    if ui.button("W").on_hover_text("Descending rating ordering").clicked() {
                        self.central_filtering.filter_watched = !self.central_filtering.filter_watched;
                        self.central_draw_list_update();
                    }
                });
            });

            ui.vertical_centered_justified(|ui| {
                // Maybe you could switch between "Search by tags" and "Search title"?
                let search_field = egui::TextEdit::singleline(&mut self.searched_string)
                    .min_size(Vec2::new(20.0, 0.0))
                    .hint_text("Find your movie / series");

                let response = ui.add(search_field);
                if response.changed() {
                    self.central_draw_list_update();
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                self.render_central_panel_entries(ctx, ui);
            });
        });
    }

    // TODO: List entries could also be draggable?
    fn render_central_panel_entries(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        for i in 0..self.central_draw_list.len() {
            // HACK: Workaround for egui junky-ness.
            //     We are using indexes because early returning after clicking the "delete" button causes
            //     the scrollbar to flicker. This ensures then we never go out of bounds after removing an
            //     item from central_draw_list and hence changing its length.
            if i >= self.central_draw_list.len() {
                break;
            }

            let entry_size = Vec2::new(ui.available_width(), 32.0);
            let (entry_rect, entry_response) = ui.allocate_exact_size(entry_size, egui::Sense::click());

            if !ui.is_rect_visible(entry_rect) {
                continue;
            }

            let mut entry_selected = self.central_draw_list[i].is_selected(&self.selected_entry);

            if entry_response.clicked() {
                let entry_id = self.central_draw_list[i].production_id;
                self.central_list_handle_selection(entry_id, entry_selected);
                entry_selected = !entry_selected;
            }

            let entry_hovered = if let Some(pos) = ctx.pointer_latest_pos() {
                entry_response.rect.contains(pos) && ui.ui_contains_pointer()
            } else {
                false
            };

            let entry_stroke = if entry_hovered {
                egui::Stroke::new(1.0, egui::Color32::from_gray(150))
            } else {
                egui::Stroke::NONE
            };

            let entry_background = if entry_selected {
                egui::Color32::from_gray(55)
            } else {
                egui::Color32::TRANSPARENT
            };

            // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
            ui.painter().rect(entry_rect, 1.0, entry_background, entry_stroke);

            let poster_pos = entry_rect.min + Vec2::new(3.0, 3.0);
            let poster_size = Vec2::new(20.0, 28.0);
            let poster_rect = Rect::from_min_size(poster_pos, poster_size);

            let poster = if let Some(ref path) = self.central_draw_list[i].poster_path {
                let image_url = TheMovieDB::get_full_poster_url(path, Width::W300);
                egui::Image::new(image_url)
            } else {
                let image_source = include_image!("../res/no_image.png");
                // let image_source = include_image!("../res/image_unavailable.svg");
                egui::Image::new(image_source)
            };

            poster.paint_at(ui, poster_rect);

            let title_font_pos = entry_rect.min + Vec2::new(32.0, entry_rect.height() / 2.0);
            let title_font_id = egui::FontId::new(12.0, eframe::epaint::FontFamily::Proportional);
            ui.painter().text(
                title_font_pos,
                egui::Align2::LEFT_CENTER,
                &self.central_draw_list[i].name,
                title_font_id,
                egui::Color32::GRAY,
            );

            if !entry_hovered {
                continue;
            }

            // Maybe this logic could be extracted or maybe a custom widget could be created instead?

            { // Drawing and handling the "delete entry" button.
                let pos = Pos2::new(entry_rect.max.x, entry_rect.min.y) - Vec2::new(30.0, -5.0);
                let size = Vec2::new(entry_rect.height() - 10.0, entry_rect.height() - 10.0);
                let rect = Rect::from_min_size(pos, size);

                let button = ui.interact(rect, egui::Id::new("central_entry_bin_btn"), egui::Sense::click());

                if button.is_pointer_button_down_on() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::RED, egui::Stroke::NONE);
                } else if button.hovered() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::LIGHT_RED, egui::Stroke::NONE);
                } else {
                    ui.painter().rect(rect, 6.0, egui::Color32::GRAY, egui::Stroke::NONE);
                }

                let icon_pos = rect.min + Vec2::new(rect.width() / 2.0 + 1.0, rect.height() / 2.0 + 1.0);
                let icon_id = egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional);

                ui.painter().text(icon_pos, egui::Align2::CENTER_CENTER, "🗑", icon_id, egui::Color32::BLACK);

                if button.clicked() {
                    self.central_list_remove_entry(self.central_draw_list[i].production_id);
                }
            }

            { // Drawing and handling the "mark favorite" button.
                let pos = Pos2::new(entry_rect.max.x, entry_rect.min.y) - Vec2::new(58.0, -5.0);
                let size = Vec2::new(entry_rect.height() - 10.0, entry_rect.height() - 10.0);
                let rect = Rect::from_min_size(pos, size);

                let button = ui.interact(rect, egui::Id::new("central_entry_fav_btn"), egui::Sense::click());

                if button.is_pointer_button_down_on() || self.central_draw_list[i].favorite {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::YELLOW, egui::Stroke::NONE);
                } else if button.hovered() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::LIGHT_YELLOW, egui::Stroke::NONE);
                } else {
                    ui.painter().rect(rect, 6.0, egui::Color32::GRAY, egui::Stroke::NONE);
                }

                let icon_pos = rect.min + Vec2::new(rect.width() / 2.0, rect.height() / 2.0 - 1.0);
                let icon_id = egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional);

                ui.painter().text(icon_pos, egui::Align2::CENTER_CENTER, "⭐", icon_id, egui::Color32::BLACK);

                if button.clicked() {
                    self.central_list_mark_favorite(self.central_draw_list[i].production_id);
                }
            }

            { // Drawing and handling the "mark watched" button.
                let pos = Pos2::new(entry_rect.max.x, entry_rect.min.y) - Vec2::new(86.0, -5.0);
                let size = Vec2::new(entry_rect.height() - 10.0, entry_rect.height() - 10.0);
                let rect = Rect::from_min_size(pos, size);

                let button = ui.interact(rect, egui::Id::new("central_entry_watch_btn"), egui::Sense::click());

                if button.is_pointer_button_down_on() || self.central_draw_list[i].watched {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::GREEN, egui::Stroke::NONE);
                } else if button.hovered() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::LIGHT_GREEN, egui::Stroke::NONE);
                } else {
                    ui.painter().rect(rect, 6.0, egui::Color32::GRAY, egui::Stroke::NONE);
                }

                let icon_pos = rect.min + Vec2::new(rect.width() / 2.0, rect.height() / 2.0 - 1.0);
                let icon_id = egui::FontId::new(18.0, eframe::epaint::FontFamily::Proportional);

                ui.painter().text(icon_pos, egui::Align2::CENTER_CENTER, "W", icon_id, egui::Color32::BLACK);

                if button.clicked() {
                    self.central_list_mark_watched(self.central_draw_list[i].production_id);
                }
            }

            //
            // TODO: Remove up and down button, replace them with drag and drop
            //       Add button that marks an entry as watched (clock icon).
            //
            { // Drawing and handling the "move down" button.
                let pos = Pos2::new(entry_rect.max.x, entry_rect.min.y) - Vec2::new(116.0, -5.0);
                let size = Vec2::new(entry_rect.height() - 10.0, entry_rect.height() - 10.0);
                let rect = Rect::from_min_size(pos, size);

                let button = ui.interact(rect, egui::Id::new("central_entry_down_btn"), egui::Sense::click());

                if button.is_pointer_button_down_on() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::BLUE, egui::Stroke::NONE);
                } else if button.hovered() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::LIGHT_BLUE, egui::Stroke::NONE);
                } else {
                    ui.painter().rect(rect, 6.0, egui::Color32::GRAY, egui::Stroke::NONE);
                }

                let icon_pos = rect.min + Vec2::new(rect.width() / 2.0, rect.height() / 2.0 - 1.0);
                let icon_id = egui::FontId::new(16.0, eframe::epaint::FontFamily::Proportional);

                ui.painter().text(icon_pos, egui::Align2::CENTER_CENTER, "🔻", icon_id, egui::Color32::BLACK);

                if button.clicked() {
                    self.central_user_list_move_down(i);
                }
            }

            { // Drawing and handling the "mark favorite" button.
                let pos = Pos2::new(entry_rect.max.x, entry_rect.min.y) - Vec2::new(148.0, -5.0);
                let size = Vec2::new(entry_rect.height() - 10.0, entry_rect.height() - 10.0);
                let rect = Rect::from_min_size(pos, size);

                let button = ui.interact(rect, egui::Id::new("central_entry_up_btn"), egui::Sense::click());

                if button.is_pointer_button_down_on() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::BLUE, egui::Stroke::NONE);
                } else if button.hovered() {
                    let rect = rect.expand(1.0);
                    ui.painter().rect(rect, 6.0, egui::Color32::LIGHT_BLUE, egui::Stroke::NONE);
                } else {
                    ui.painter().rect(rect, 6.0, egui::Color32::GRAY, egui::Stroke::NONE);
                }

                let icon_pos = rect.min + Vec2::new(rect.width() / 2.0, rect.height() / 2.0 - 1.0);
                let icon_id = egui::FontId::new(16.0, eframe::epaint::FontFamily::Proportional);

                ui.painter().text(icon_pos, egui::Align2::CENTER_CENTER, "🔺", icon_id, egui::Color32::BLACK);

                if button.clicked() {
                    self.central_user_list_move_up(i);
                }
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

                let display = if self.selection.season.is_some() {
                    format!("S{}", self.selection.season())
                } else {
                    "None".to_string()
                };

                let before_render_season = self.selection.season.unwrap_or(0);
                egui::ComboBox::from_label("Select season!")
                    .selected_text(display)
                    .show_ui(ui, |ui| {
                        for i in 1..=series.number_of_seasons {
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

                    let all_episodes = if series.has_specials() {
                        series.seasons[season_num].episode_count
                    } else {
                        series.seasons[season_num - 1].episode_count
                    };

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

            ui.separator();
            ui.add_space(8.0);

            // lots of duplicates
            // TODO: Implement methods for notes on seasons and episodes for safer access
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
                    let rating = match self.selection.season {
                        Some(season_num) => &mut user_series.season_notes[season_num as usize - 1].user_rating,
                        None => &mut user_series.user_rating,
                    };
                    ui.add(
                        egui::DragValue::new(rating)
                            .speed(0.1)
                            .clamp_range(RangeInclusive::new(0.0, 10.0)),
                    );
                    ui.label("/ 10")
                });
                ui.add_space(8.0);
                if let Some(episode_num) = self.selection.episode {
                    let season_num = self.selection.season();

                    // NOTE: Format every frame. BAD! We need to cache it.
                    ui.label(format!("S{season_num} E{episode_num} notes:"));
                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                        let season_notes = &mut user_series.season_notes[season_num as usize - 1];
                        ui.text_edit_multiline(&mut season_notes.episode_notes[episode_num as usize - 1]);
                    });
                    return;
                }

                if let Some(season_num) = self.selection.season {
                    // NOTE: Format every frame. BAD! We need to cache it.
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

                    if ui.button("Load data from file").clicked() {
                        todo!();
                    }

                    let migrate_data = ui.add_enabled(true, egui::Button::new("Migrate data"));
                    if migrate_data.clicked() {
                        for series in &self.user_series {
                            self.prod_positions.push(ProdEntry::new(false, series.series.id));
                        }

                        for movie in &self.user_movies {
                            self.prod_positions.push(ProdEntry::new(true, movie.movie.id));
                        }

                        // unreachable!("There is nothing to migrate. You shouldn't be able to click this by the way...");
                    }

                    if ui.button("Save config").clicked() {
                        if self.config.validate_access_token() {
                            self.config.save("res/config.json");
                        } else {
                            self.toasts.add(Toast{
                                kind: ToastKind::Error,
                                text: "Read access token should contain 211 characters and include 2 dots".into(),
                                options: ToastOptions::default()
                                    .duration_in_seconds(3.5)
                                    .show_progress(true)
                                    .show_icon(true),
                            });
                        }
                    }

                    if ui.button("Load config").clicked() {
                        self.config = Config::load("res/config.json");
                    }
                });

                ui.menu_button("View", |ui| {
                    // why is this so laggy?
                    // skill issue...
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
                            - [x] Enable/Disable
                            - [ ] Update interval
                        - [ ] Sync
                            - [ ] Sync to server
                            - [ ] Sync from server
                        - [x] Enable/Disable local caching
                        - [x] Set tmdb token
                        - [x] Set default browser
                        - [ ] Autoload on startup
                        - [ ] Autosave on exit
                    */

                    ui.menu_button("Set TMDB token", |ui| {
                        // NOTE: This won't work because we are passing the API key to TheMovieDb struct
                        ui.text_edit_singleline(&mut self.config.access_token);
                    });

                    ui.menu_button("Set default browser", |ui| {
                        ui.text_edit_singleline(&mut self.config.browser_name);
                    });

                    let autosave_label = if self.config.autosave {
                        "Disable auto-save"
                    } else {
                        "Enable auto-save"
                    };
                    if ui.button(autosave_label).clicked() {
                        self.config.autosave = !self.config.autosave;
                    }

                    let caching_label = if self.config.enable_cache {
                        "Disable caching"
                    } else {
                        "Enable caching"
                    };
                    if ui.button(caching_label).clicked() {
                        self.config.enable_cache = !self.config.enable_cache;
                    }

                    if ui.button("Sync").clicked() {
                        todo!()
                    }
                });

                ui.menu_button("About", |_| {});
                ui.menu_button("License", |_| {
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
                    self.add_movie(movie.clone());
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
                    let resource = TheMovieDB::get_full_poster_url(poster, Width::Original);
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
            //       (it still is very bad...)
            if let hash_map::Entry::Vacant(e) = self.description_cache.entry(movie.id) {
                let slice = &movie.overview.as_bytes()[..200];
                let description = format!("{}...", String::from_utf8_lossy(slice).trim());
                ui.label(&description);
                e.insert(description);
            } else {
                let description = self.description_cache.get(&movie.id).expect("Not cached");
                ui.label(description);
            }
        } else {
            ui.label(&movie.overview);
        };

        ui.separator();
    }

    fn draw_series_entry(&mut self, ui: &mut Ui, series: &SearchedSeries) {
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
                    let details = self.movie_db.get_series_details_now(series.id);
                    let new_data = Series::from(series, details);
                    self.add_series(new_data);
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
                    let resource = TheMovieDB::get_full_poster_url(&poster, Width::Original);
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
            //       (it still is very bad...)
            if let hash_map::Entry::Vacant(e) = self.description_cache.entry(series.id + 1) {
                let slice = &series.overview.as_bytes()[..200];
                let description = format!("{}...", String::from_utf8_lossy(slice).trim());
                ui.label(&description);
                e.insert(description);
            } else {
                let description = self.description_cache.get(&(series.id + 1)).expect("Not cached");
                ui.label(description);
            }
        } else {
            ui.label(&series.overview);
        };

        ui.separator();
    }

    fn production_grid(&mut self, ui: &mut Ui, searched: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if searched {
                ui.scroll_to_cursor(Some(Align::Center));
            }

            if let Some(mut productions) = self.fetch_productions_job.poll_owned() {
                self.sort_productions_by_popularity(&mut productions);
                let productions: Rc<[Production]> = productions.into();
                self.search_productions = Some(productions);
            }

            let Some(productions) = self.search_productions.clone() else {
                return;
            };

            for prod in &*productions {
                match prod {
                    Production::Movie(ref movie) => self.draw_movie_entry(ui, movie),
                    Production::SearchedSeries(ref series) => self.draw_series_entry(ui, series),
                }
            }
        });
    }

    fn sort_productions_by_popularity(&mut self, productions: &mut Vec<Production>) {
        productions.sort_by(|e1, e2| {
            let pop1 = match e1 {
                Production::Movie(ref movie1) =>  movie1.popularity,
                Production::SearchedSeries(ref series1) => series1.popularity
            };
            let pop2 = match e2 {
                Production::Movie(ref movie2) => movie2.popularity,
                Production::SearchedSeries(ref series2) => series2.popularity
            };
            pop2.partial_cmp(&pop1).unwrap()
        });
    }
}

struct Selection {
    //index into user movies / user series, depending on selected_entry
    index: Option<usize>,
    season: Option<u32>,  //cannot be 0
    episode: Option<u32>, //cannot be 0
}

#[allow(dead_code)]
impl Selection {
    pub fn new() -> Self {
        Self {
            index: None,
            season: None,
            episode: None,
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


