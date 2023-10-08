use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TVShow {
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

impl TVShow {
    pub fn parse(json: &str) -> TVShow{
        serde_json::from_str(json).expect("Failed to deserialize a TVShow object")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Movie{
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

impl Movie{
    pub fn parse(json: &str) -> Movie{
        serde_json::from_str(json).expect("Failed to deserialize a Movie object")
    }
}

pub enum Production{
    Film(Movie),
    Series(TVShow)
}