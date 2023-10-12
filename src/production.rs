use serde::{Deserialize, Serialize};

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

impl Series {
    pub fn parse(json: &str) -> Series {
        serde_json::from_str(json).expect("Failed to deserialize a TVShow object")
    }
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

impl Movie {
    pub fn parse(json: &str) -> Movie {
        serde_json::from_str(json).expect("Failed to deserialize a Movie object")
    }
}

pub enum Production {
    Movie(Movie),
    Series(Series),
}
