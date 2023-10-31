use std::fs::File;
use std::io::BufReader;
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
    pub season_notes: SeasonNotes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonNotes{
    pub note: String,
    pub episode_notes: Vec<String>
}

//pass Vec<UserMovie>
pub fn serialize_user_productions(user_series: Vec<UserSeries>, user_movies: Vec<UserMovie>){
    let john = json!({
        "series": user_series,
        "movies": user_movies
    });
    let serialized_json = serde_json::to_string(&john).expect("Failed to serialize JSON");

    let _path = "../user_prod.json";
    // Write to a file, or write to a temp file then move files.
    println!("{}", serialized_json);
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
**/
