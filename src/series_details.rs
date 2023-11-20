use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
//we need this struct in order to know how many seasons a series has (are unreleased seasons included?)
pub struct SeriesDetails {
    pub number_of_seasons: u32,
    pub number_of_episodes: u32,
    pub status: String, //is finished?
    //pub episode_run_time: Vec<u32>, this is broken
    pub seasons: Vec<Season>,
}

impl SeriesDetails {
    pub fn has_specials(&self) -> bool {
        self.seasons[0].season_number == 0
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
//represents episodes of one season, not the entire series (shouldn't include what Season already has)
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

        Runtime {
            minutes,
            hours
        }
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
