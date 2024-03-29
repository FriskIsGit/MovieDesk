use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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
    pub number_of_seasons: u32,
    pub number_of_episodes: u32,
    pub status: String,
    pub seasons: Vec<Season>,
}
impl Series {
    pub fn from(series: &SearchedSeries, details: SeriesDetails) -> Self {
        Self {
            id: series.id,
            name: series.name.clone(),
            original_language: series.original_language.clone(),
            overview: series.overview.clone(),
            popularity: series.popularity,
            poster_path: series.poster_path.clone(),
            first_air_date: series.first_air_date.clone(),
            vote_average: series.vote_average,
            adult: series.adult,
            number_of_seasons: details.number_of_seasons,
            number_of_episodes: details.number_of_episodes,
            status: details.status,
            seasons: details.seasons,
        }
    }

    pub fn has_specials(&self) -> bool {
        self.seasons[0].season_number == 0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchedSeries {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSeries {
    pub series: Series,
    pub user_rating: f32,
    pub note: String,
    pub season_notes: Vec<SeasonNotes>,
    pub watched: bool,
    pub favorite: bool
}

impl UserSeries {
    pub fn new(series: Series) -> Self {
        let mut notes = Vec::with_capacity(series.seasons.len());
        for season in &series.seasons {
            let season_notes = SeasonNotes::empty(season.episode_count as usize);
            notes.push(season_notes);
        }

        Self {
            series,
            note: String::new(),
            user_rating: 0.0,
            season_notes: notes,
            watched: false,
            favorite: false,
        }
    }
    // notes for specials end up at the end of the vector
    pub fn season_note(&mut self, season_num: u32) -> &mut SeasonNotes {
        &mut self.season_notes[season_num as usize - 1]
    }
}

#[derive(Debug, Serialize, Deserialize)]
// We need this struct in order to know how many seasons a series has (are unreleased seasons included?)
pub struct SeriesDetails {
    pub number_of_seasons: u32,
    pub number_of_episodes: u32,
    pub status: String, //is finished?
    //pub episode_run_time: Vec<u32>, this is broken
    pub seasons: Vec<Season>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    pub air_date: Option<String>,
    pub episode_count: u32,
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub poster_path: Option<String>,
    pub season_number: u32,
    pub vote_average: f32,
}

//--------------------------------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
// Represents episodes of one season, not the entire series (shouldn't include what Season already has)
pub struct SeasonDetails {
    pub id: u32,
    pub season_number: u32,
    pub name: String,
    pub air_date: Option<String>,
    pub episodes: Vec<Episode>,
}

impl SeasonDetails {
    pub fn runtime(&self) -> Runtime {
        let minutes = self.episodes.iter().filter_map(|ep| ep.runtime).sum();
        let hours = minutes as f32 / 60.0;

        Runtime { minutes, hours }
    }
}

pub struct Runtime {
    minutes: u32,
    hours: f32,
}

impl Display for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}min {:.2}hr", self.minutes, self.hours)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub episode_number: u32,
    pub name: String,
    pub overview: String,
    pub runtime: Option<u32>,
    pub vote_average: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonNotes {
    pub note: String,
    pub user_rating: f32,
    pub episode_notes: Vec<String>,
}

impl SeasonNotes {
    pub fn empty(episode_count: usize) -> Self {
        let episode_notes = vec![String::new(); episode_count];
        Self {
            note: "".into(),
            user_rating: 0.0,
            episode_notes,
        }
    }
    pub fn episode_note(&mut self, episode_num: u32) -> &mut String {
        &mut self.episode_notes[episode_num as usize - 1]
    }
    pub fn ensure_length(&mut self, episode_count: usize) {
        if episode_count <= self.episode_notes.len() {
            return
        }
        let fill = episode_count - self.episode_notes.len();
        for _ in 0..fill {
            self.episode_notes.push(String::new());
        }
    }
}
