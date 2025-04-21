use std::collections::{HashMap, HashSet};
use crate::movies::{Movie, UserMovie};
use crate::series::{SearchedSeries, SeasonNotes, UserSeries};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufReader, Write};

pub enum Production {
    Movie(Movie),
    SearchedSeries(SearchedSeries),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionIds {
    pub id: u32,
    pub facebook_id: Option<String>,
    pub freebase_id: Option<String>,
    pub freebase_mid: Option<String>,
    pub imdb_id: Option<String>,
    pub instagram_id: Option<String>,
    pub tvdb_id: Option<u32>,
    pub tvrage_id: Option<u32>,
    pub twitter_id: Option<String>,
    pub wikidata_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trailer {
    pub name: String,
    pub key: String,
    pub published_at: String,
    pub site: String,
    pub size: u32,
    pub official: bool,
}

impl Trailer {
    pub fn youtube_url(&self) -> String {
        format!("https://youtube.com/watch?v={}", self.key)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keyword {
    id: usize,
    name: String,
}

#[derive(Debug)]
pub struct UserData {
    pub user_series: Vec<UserSeries>,
    pub user_movies: Vec<UserMovie>,
    pub prod_positions: Vec<ProdEntry>,
}
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct ProdEntry {
    pub is_movie: bool,
    pub id: u32,
}
impl ProdEntry {
    pub fn new(is_movie: bool, id: u32) -> Self{
        Self { is_movie, id }
    }
}

pub fn serialize_user_productions(user_series: &[UserSeries], user_movies: &[UserMovie], prod_positions: &[ProdEntry]) -> Result<(), String> {
    let john = json!({
        "series": user_series,
        "movies": user_movies,
        "positions": prod_positions,
    });
    let serialized_json = serde_json::to_string(&john).expect("Failed to serialize JSON");
    let temp_path = "res/user_prod_temp.json";
    let mut file = match File::create(temp_path) {
        Ok(file_handle) => file_handle,
        Err(err) => return Err(err.to_string()),
    };

    if let Err(err) = file.write(serialized_json.as_bytes()) {
        return Err(err.to_string());
    }

    // Write to a file, or write to a temp file then move files.
    let path = "res/user_prod.json";
    match std::fs::rename(temp_path, path) {
        Err(err) => Err(err.to_string()),
        Ok(_) => Ok(()),
    }
}

pub fn deserialize_user_productions(path: Option<String>) -> Result<UserData, String> {
    let path = match path {
        Some(s) => s,
        None => "res/user_prod.json".into(),
    };
    let file = match File::open(path) {
        Ok(file_handle) => file_handle,
        Err(err) => return Err(err.to_string()),
    };
    let reader = BufReader::new(file);
    let mut json: Value = serde_json::from_reader(reader).expect("Failed on read from memory");
    let series_arr = json["series"].take();
    let movies_arr = json["movies"].take();
    let user_series = match serde_json::from_value(series_arr) {
        Ok(vec_value) => vec_value,
        Err(err) => return Err(err.to_string()),
    };
    let user_movies = match serde_json::from_value(movies_arr) {
        Ok(vec_value) => vec_value,
        Err(err) => return Err(err.to_string()),
    };
    let positions_arr = json["positions"].take();
    // Allow null for now
    let prod_positions = if Value::Null == positions_arr {
        vec![]
    } else {
        match serde_json::from_value(positions_arr){
            Ok(vec_value) => vec_value,
            Err(err) => return Err(err.to_string()),
        }
    };

    let data = UserData {
        user_series,
        user_movies,
        prod_positions
    };
    Ok(data)
}

type Changes = usize;
pub fn fix_data_integrity(user_series: &mut Vec<UserSeries>,
                          user_movies: &mut Vec<UserMovie>,
                          prod_positions: &mut Vec<ProdEntry>) -> Changes {
    let mut changes = 0;
    let prod_len = user_series.len() + user_movies.len();
    let positions_len = prod_positions.len();
    // #1 Production positions missing
    if prod_len != positions_len {
        if prod_len > positions_len {
            let mut position_ids: HashSet<u32> = HashSet::new();
            for position in prod_positions.iter() {
                position_ids.insert(position.id);
            }
            for u_series in user_series.iter() {
                if !position_ids.contains(&u_series.series.id) {
                    let prod_entry = ProdEntry::new(false, u_series.series.id);
                    prod_positions.push(prod_entry);
                    changes += 1;
                }
            }
            for u_movie in user_movies {
                if !position_ids.contains(&u_movie.movie.id) {
                    let prod_entry = ProdEntry::new(true, u_movie.movie.id);
                    prod_positions.push(prod_entry);
                    changes += 1;
                }
            }
        } else {
            // TODO delete prod entries which don't map to any productions
        }
    }

    // #2 Ensure notes lengths
    for u_series in user_series {
        let mut episode_counts = Vec::with_capacity(u_series.series.seasons.len());
        let first_season_index = if u_series.series.has_specials() { 1 } else { 0 };
        let seasons = &u_series.series.seasons;
        // println!("Checking {} | seasons: {}", u_series.series.name, u_series.series.number_of_seasons);
        for i in first_season_index..seasons.len() {
            let season = &seasons[i];
            episode_counts.push(season.episode_count as usize)
        }
        let story_seasons = u_series.series.number_of_seasons; // specials aren't included
        for i in 0..story_seasons {
            let i_index = i as usize;
            let season_note = &mut u_series.season_notes[i_index];
            if season_note.episode_notes.len() < episode_counts[i_index] {
                season_note.ensure_length(episode_counts[i_index]);
                changes += 1;
            }
        }
    }
    changes
}

// rating conflicts will not be merged
pub fn merge_data(user_series: &mut Vec<UserSeries>,
                          user_movies: &mut Vec<UserMovie>,
                          prod_positions: &mut Vec<ProdEntry>, merge_path: &str) -> Result<(), String> {
    let outcome = deserialize_user_productions(Some(merge_path.into()));
    let Ok(mut other_data) = outcome else {
        return Err(outcome.unwrap_err());
    };

    // Common movies merge
    for mov in user_movies.iter_mut() {
        for other_movie in &other_data.user_movies {
            if mov.movie.id != other_movie.movie.id {
                continue
            }

            println!("Merging movie: {}", mov.movie.title);
            mov.favorite = mov.favorite || other_movie.favorite;
            mov.watched = mov.watched || other_movie.watched;
            merge_strings(&mut mov.note, &other_movie.note);
            mov.user_rating = pick_with_value(mov.user_rating, other_movie.user_rating);
            break
        }
    }

    // Common series merge
    for series in user_series.iter_mut() {
        for other_series in &other_data.user_series {
            if series.series.id != other_series.series.id {
                continue
            }
            println!("Merging series: {}", series.series.name);
            series.favorite = series.favorite || other_series.favorite;
            series.watched = series.watched || other_series.watched;
            merge_strings(&mut series.note, &other_series.note);
            series.user_rating = pick_with_value(series.user_rating, other_series.user_rating);
            merge_season_notes(&mut series.season_notes, &other_series.season_notes);
            break
        }
    }

    // Series that are only present in the other data set, consumed into the hashmap
    let mut new_series: HashMap<u32, UserSeries> = HashMap::new();
    while let Some(u_series) = other_data.user_series.pop() {
        new_series.insert(u_series.series.id, u_series);
    }
    for u_series in user_series.iter() {
        if new_series.contains_key(&u_series.series.id) {
            new_series.remove(&u_series.series.id);
        }
    }

    let new_series_ids: Vec<u32> = new_series.keys().cloned().collect();
    for id in new_series_ids.iter() {
        let Some(series) = new_series.remove(id) else {
            continue
        };
        let prod_entry = ProdEntry::new(false, *id);
        println!("Adding {}", series.series.name);
        prod_positions.push(prod_entry);
        user_series.push(series);
    }

    // Movies that are only present in the other data set, consumed into the hashmap
    let mut new_movies: HashMap<u32, UserMovie> = HashMap::new();
    while let Some(u_movie) = other_data.user_movies.pop() {
        new_movies.insert(u_movie.movie.id, u_movie);
    }
    for u_movie in user_movies.iter() {
        if new_movies.contains_key(&u_movie.movie.id) {
            new_movies.remove(&u_movie.movie.id);
        }
    }

    let new_movies_ids: Vec<u32> = new_movies.keys().cloned().collect();
    for id in new_movies_ids.iter() {
        let Some(movie) = new_movies.remove(id) else {
            continue
        };
        let prod_entry = ProdEntry::new(true, *id);
        println!("Adding {}", movie.movie.title);
        prod_positions.push(prod_entry);
        user_movies.push(movie);
    }
    Ok(())
}

fn merge_season_notes(notes: &mut Vec<SeasonNotes>, other_notes: &Vec<SeasonNotes>) {
    let len = notes.len();
    if len != other_notes.len() {
        eprintln!("Refusing to merge because of different note lengths");
        return
    }
    for i in 0..len {
        let season = &mut notes[i];
        let other_season = &other_notes[i];
        merge_strings(&mut season.note, &other_season.note);
        season.user_rating = pick_with_value(season.user_rating, other_season.user_rating);
        if season.episode_notes.len() != other_season.episode_notes.len() {
            eprintln!("Refusing to merge because of different episode lengths: {}, {}",
                      season.episode_notes.len(),
                      other_season.episode_notes.len(),
            );
            continue
        }
        let episode_count = season.episode_notes.len();
        for j in 0..episode_count {
            merge_strings(&mut season.episode_notes[j], &other_season.episode_notes[j]);
        }
    }
}

fn pick_with_value(rating: f32, other_rating: f32) -> f32 {
    if rating == 0.0 {
        return other_rating
    }
    return rating
}

pub fn merge_strings(merged: &mut String, with: &str) {
    if merged.is_empty() {
        merged.push_str(&with);
        return
    }
    if with.is_empty() || merged.starts_with(with) {
        return
    }

    let find_result = with.find(&*merged);
    if find_result.is_some() && find_result.unwrap() == 0 {
        merged.clear();
        merged.push_str(with)
    } else {
        merged.push_str("\n>>>>>>>>\n");
        merged.push_str(with);
    }
}

/*
Serialization:
user_prod.json
{
    "series":[
        {UserSeries}
        {UserSeries}
    ]
    "movies":[
        {UserMovie}
        {UserMovie}
    ]
    "positions":[
        {ProdEntry}
        {ProdEntry}
    ]
}
*/

type ProductionId = u32;

#[derive(Default, Copy, Clone)]
pub enum EntryType {
    Movie(ProductionId),
    Series(ProductionId),
    #[default]
    None,
}

// NOTE: Central list could hold UserProduction instead that is displayed on top of the right panel maybe?
#[derive(Clone)]
pub struct ListEntry {
    pub production_id: EntryType,

    // NOTE: Those below could be references to the item from the UserProduction?
    pub name: String,
    pub poster_path: Option<String>, // Shouldn't be an option, should always have a fallback image btw.
    pub rating: f32,

    pub favorite: bool,
    pub watched: bool,
}

impl ListEntry {
    pub fn from_movie(user_movie: &UserMovie) -> Self {
        let movie = &user_movie.movie;
        Self {
            production_id: EntryType::Movie(movie.id),

            name: movie.title.clone(),
            poster_path: movie.poster_path.clone(),
            rating: movie.vote_average,

            favorite: user_movie.favorite,
            watched: user_movie.watched,
        }
    }

    pub fn from_series(user_series: &UserSeries) -> Self {
        let series = &user_series.series;
        Self {
            production_id: EntryType::Series(series.id),

            name: series.name.clone(),
            poster_path: series.poster_path.clone(),
            rating: series.vote_average,

            favorite: user_series.favorite,
            watched: user_series.watched,
        }
    }

    pub fn is_selected(&self, entry: &EntryType) -> bool {
        match entry {
            EntryType::Movie(selected_id) => {
                let EntryType::Movie(list_entry_id) = &self.production_id else {
                    return false;
                };
                selected_id == list_entry_id
            }
            EntryType::Series(selected_id) => {
                let EntryType::Series(list_entry_id) = &self.production_id else {
                    return false;
                };
                selected_id == list_entry_id
            }
            EntryType::None => false,
        }
    }
}

pub struct ListFiltering {
    pub filter_favorites: bool,
    pub filter_watched: bool,
    pub filter_to_watch: bool,
}

impl ListFiltering {
    pub fn new() -> Self {
        Self {
            filter_favorites: false,
            filter_watched:   false,
            filter_to_watch:  false,
        }
    }
}

pub enum ListOrdering {
    UserDefined,
    Alphabetic,
    RatingAscending,
    RatingDescending,
}
