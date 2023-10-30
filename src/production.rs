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

pub struct UserProduction {
    pub production: Production,
    pub user_rating: f32,
    pub user_note: String,
}
