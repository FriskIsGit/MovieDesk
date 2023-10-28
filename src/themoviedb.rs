use crate::config::Config;
use crate::production::{Movie, Production, Series};
use crate::series_details::{SeasonDetails, SeriesDetails};
use serde_json::Value;
use std::time::Duration;
use ureq;
use ureq::{Agent, AgentBuilder};

const SEARCH_MULTI_URL: &str = "https://api.themoviedb.org/3/search/multi";
const SERIES_DETAILS_URL: &str = "https://api.themoviedb.org/3/tv/"; //{series_id}
const IMAGE_URL: &str = "https://image.tmdb.org/t/p/";

#[allow(dead_code)]
pub enum Width {
    W200,
    W300,
    W400,
    W500,
    ORIGINAL,
}

pub struct TheMovieDB {
    api_key: String,
    agent: Agent,
    use_cache: bool,
    query_to_prod: VecMap<String, Vec<Production>>,
    // cache object outputs to avoid making multiple requests for the same data
}

impl TheMovieDB {
    pub fn new(key: String, use_cache: bool) -> Self {
        Self {
            api_key: key,
            agent: AgentBuilder::new().timeout(Duration::from_secs(15)).build(),
            use_cache,
            query_to_prod: VecMap::new(),
        }
    }

    fn new_authorized_get(&self, url: &str) -> ureq::Request {
        let request = self
            .agent
            .get(url)
            .set("Accept", "application/json")
            .set("Authorization", &format!("Bearer {}", self.api_key));
        request
    }

    pub fn search_production(&self, query: &str) -> Vec<Production> {
        let url = format!("{SEARCH_MULTI_URL}?query={}&include_adult={}", query, true);
        let request = self.new_authorized_get(&url);

        println!("Executing request..");
        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let status = response.status();
        if status != 200 {
            println!("status: {}", status)
        }

        let json_response = response.into_string().unwrap().to_owned();

        let payload: Value = serde_json::from_str(json_response.as_str()).unwrap();
        let arr: Value = payload["results"].to_owned();
        if !arr.is_array() {
            eprintln!("Results are not in an array");
            return Vec::new();
        }
        let list = arr.as_array().unwrap();
        let mut productions: Vec<Production> = Vec::with_capacity(list.len());
        for prod_obj in list {
            let media_type = prod_obj["media_type"].to_owned();
            if media_type == "tv" {
                let series = Series::parse(prod_obj.to_string().as_str());
                productions.push(Production::Series(series));
            } else if media_type == "movie" {
                let movie = Movie::parse(prod_obj.to_string().as_str());
                productions.push(Production::Movie(movie));
            }
        }
        productions
    }

    pub fn get_full_poster_url(poster: &String, width: Width) -> String {
        let size = match width {
            Width::W200 => "w200",
            Width::W300 => "w300",
            Width::W400 => "w400",
            Width::W500 => "w500",
            Width::ORIGINAL => "original",
        };
        format!("{IMAGE_URL}{size}{poster}")
    }

    pub fn get_series_details(&self, id: u32) -> SeriesDetails {
        let url = format!("{SERIES_DETAILS_URL}{id}");
        let request = self.new_authorized_get(&url);

        println!("Executing request..");

        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let json = response.into_string().unwrap();
        SeriesDetails::parse(json.as_str())
    }

    pub fn get_season_details(&self, series_id: u32, season_number: u32) -> SeasonDetails {
        let url = format!("{SERIES_DETAILS_URL}{series_id}/season/{season_number}");
        let request = self.new_authorized_get(&url);

        println!("Executing request..");
        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let json = response.into_string().unwrap();
        SeasonDetails::parse(json.as_str())
    }

    pub fn download_resource(&self, resource_url: &str) -> Vec<u8> {
        let request = self.agent.get(resource_url);
        println!("Executing request..");
        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let mut buf = Vec::with_capacity(4096);
        let bytes_written = response.into_reader().read_to_end(&mut buf).unwrap();
        println!("bytes written {}", bytes_written);
        return buf;
    }
}

struct VecMap<K, V>{
    keys_to_values: Vec<(K, V)>,
}
impl<K: PartialEq, V> VecMap<K, V> {
    pub fn new() -> VecMap<K, V>{
        Self {
            keys_to_values: vec![],
        }
    }
    pub fn put(&mut self, key: K, value: V){
        self.keys_to_values.push((key, value));
    }
    pub fn get(&self, key: K) -> Option<&V> {
        for pair in &self.keys_to_values {
            if pair.0 == key {
                return Some(&pair.1);
            }
        }
        None
    }
    pub fn remove(&mut self, key: K) {
        let mut index: i64 = -1;
        for (i, pair) in self.keys_to_values.iter().enumerate() {
            if pair.0 == key {
                index = i as i64;
                break;
            }
        }
        if index != -1 {
            self.keys_to_values.remove(index as usize);
        }
    }
    pub fn size(&self) -> usize{
        self.keys_to_values.len()
    }
}

