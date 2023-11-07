use std::fs::File;
use std::io::{BufReader, Write};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub id: u32,
    pub name: String,
    pub original_language: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub first_air_date: String,
    pub vote_average: f32,
    pub adult: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    pub title: String,
    pub original_language: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: String,
    pub vote_average: f32,
    pub adult: bool,
}

#[derive(Clone)]
pub enum Production {
    Movie(Movie),
    Series(Series),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMovie {
    pub movie: Movie,
    pub user_rating: f32,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSeries {
    pub series: Series,
    pub user_rating: f32,
    pub note: String,
    pub season_notes: Vec<SeasonNotes>
}
impl UserSeries{
    pub fn ensure_seasons(&mut self, len: usize) {
        if self.season_notes.len() >= len {
            return;
        }
        let fill = len - self.season_notes.len();
        for _ in 0..fill {
            self.season_notes.push(SeasonNotes::new());
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonNotes{
    pub note: String,
    pub episode_notes: Vec<String>
}
impl SeasonNotes {
    pub fn new() -> Self {
        Self{
            note: "".into(),
            episode_notes: Vec::new(),
        }
    }
    pub fn ensure_episodes(&mut self, len: usize) {
        if self.episode_notes.len() >= len {
            return;
        }
        let fill = len - self.episode_notes.len();
        for _ in 0..fill {
            self.episode_notes.push("".into());
        }
    }
}

pub fn serialize_user_productions(user_series: &[UserSeries], user_movies: &[UserMovie]) -> Result<(), String>{
    let john = json!({
        "series": user_series,
        "movies": user_movies
    });
    let serialized_json = serde_json::to_string(&john).expect("Failed to serialize JSON");
    let temp_path = "res/user_prod_temp.json";
    let mut file = match File::create(temp_path) {
        Ok(file_handle) => file_handle,
        Err(err) => return Err(err.to_string())
    };
    match file.write(serialized_json.as_bytes()) {
        Err(err) => return Err(err.to_string()),
        _ => {}
    };
    // Write to a file, or write to a temp file then move files.
    let path = "res/user_prod.json";
    return match std::fs::rename(temp_path, path) {
        Err(err) => Err(err.to_string()),
        Ok(_) => Ok(())
    };
}

pub fn deserialize_user_productions(path: Option<String>) -> Result<(Vec<UserSeries>, Vec<UserMovie>), String> {
    let path = match path {
        Some(s) => s,
        None => "res/user_prod.json".into(),
    };
    let file = match File::open(path) {
        Ok(file_handle) => file_handle,
        Err(err) => return Err(err.to_string())
    };
    let reader = BufReader::new(file);
    let mut json: Value = serde_json::from_reader(reader).expect("Failed on read from memory");
    let series_arr = json["series"].take();
    let movies_arr = json["movies"].take();
    let user_series = match serde_json::from_value(series_arr) {
        Ok(vec_value) => vec_value,
        Err(err) => return Err(err.to_string())
    };
    let user_movies = match serde_json::from_value(movies_arr) {
        Ok(vec_value) => vec_value,
        Err(err) => return Err(err.to_string())
    };
    Ok((user_series, user_movies))
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
}
*/