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

//convert UserProduction to UserMovie
pub struct UserProduction {
    pub production: Production,
    pub user_rating: f32,
    pub note: String,
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

//pass Vec<UserMovie>
pub fn serialize_user_productions(user_series: &Vec<UserSeries>, user_movies: &Vec<UserMovie>){
    let john = json!({
        "series": user_series,
        "movies": user_movies
    });
    let serialized_json = serde_json::to_string(&john).expect("Failed to serialize JSON");
    let temp_path = "../user_prod_temp.json";
    println!("{}", serialized_json);
    let result = File::create(temp_path.clone()).expect("Unable to create file").write(serialized_json.as_bytes());
    if result.is_err() {
        eprintln!("Unable to write")
    }
    // Write to a file, or write to a temp file then move files.
    let path = "../user_prod.json";
    std::fs::rename(temp_path, path).expect("Unable to move/rename");
}

pub fn deserialize_user_productions() -> (Vec<UserSeries>, Vec<UserMovie>){
    let path = "../user_prod.json";
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut json: Value = serde_json::from_reader(reader).expect("Failed on read from memory");
    let user_series = json["series"].take();
    let user_movies = json["movies"].take();
    (
        serde_json::from_value(user_series).expect("Failed to deserialize UserSeries"),
        serde_json::from_value(user_movies).expect("Failed to deserialize UserMovies")
    )
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
