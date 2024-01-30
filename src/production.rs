use std::collections::HashSet;
use crate::movies::{Movie, UserMovie};
use crate::series::{SearchedSeries, UserSeries};
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
