use crate::jobs::Job;
use crate::production::{Keyword, Production, ProductionIds, Trailer};
use crate::series::{SeasonDetails, SeriesDetails};
use egui::TextBuffer;
use serde_json::Value;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};
use crate::movies::MovieDetails;

const SEARCH_MULTI_URL: &str = "https://api.themoviedb.org/3/search/multi";
const SERIES_DETAILS_URL: &str = "https://api.themoviedb.org/3/tv/"; //{series_id}
const MOVIE_DETAILS_URL: &str = "https://api.themoviedb.org/3/movie/"; //{movie_id}
const IMAGE_URL: &str = "https://image.tmdb.org/t/p/";
const IMDB_TITLE: &str = "https://www.imdb.com/title/";
const IMDB_FIND: &str = "https://www.imdb.com/find/?q=";

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
    // pub use_cache: bool,
    // cache object outputs to avoid making multiple requests for the same data
    // query_to_prod: VecMap<String, Vec<Production>>,
}

impl TheMovieDB {
    pub fn new(key: String, _use_cache: bool) -> Self {
        Self {
            api_key: key,
            agent: AgentBuilder::new().timeout(Duration::from_secs(15)).build(),
            // use_cache,
            // query_to_prod: VecMap::new(),
        }
    }

    fn new_authorized_get(&self, url: &str) -> ureq::Request {
        self.agent
            .get(url)
            .set("Accept", "application/json")
            .set("Authorization", &format!("Bearer {}", self.api_key))
    }

    pub fn search_production(&mut self, query: String) -> Job<Vec<Production>> {
        let url = format!("{SEARCH_MULTI_URL}?query={query}&include_adult={}", true);
        let request = self.new_authorized_get(&url);

        Job::new(move || {
            println!("Executing request in search_production");
            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };

            let status = response.status();
            if status != 200 {
                println!("status: {}", status)
            }

            let json_response = response.into_string().unwrap().take();

            let mut payload: Value = serde_json::from_str(json_response.as_str()).unwrap();
            let mut arr: Value = payload["results"].take();
            if !arr.is_array() {
                eprintln!("Results are not in an array");
                return Vec::new();
            }
            let list = arr.as_array_mut().unwrap();
            let mut productions = Vec::with_capacity(list.len());

            for prod_obj in list {
                let media_type = &prod_obj["media_type"];
                if media_type == "tv" {
                    let searched_series = serde_json::from_value(prod_obj.take()).unwrap();
                    productions.push(Production::SearchedSeries(searched_series));
                } else if media_type == "movie" {
                    let movie = serde_json::from_value(prod_obj.take()).unwrap();
                    productions.push(Production::Movie(movie));
                }
            }

            productions
        })
    }

    pub fn get_full_poster_url(poster: &str, width: Width) -> String {
        let size = match width {
            Width::W200 => "w200",
            Width::W300 => "w300",
            Width::W400 => "w400",
            Width::W500 => "w500",
            Width::ORIGINAL => "original",
        };
        format!("{IMAGE_URL}{size}{poster}")
    }

    pub fn get_series_details(&self, id: u32) -> Job<SeriesDetails> {
        let url = format!("{SERIES_DETAILS_URL}{id}");
        let request = self.new_authorized_get(&url);

        Job::new(move || {
            println!("Executing request in get_series_details");

            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };

            serde_json::from_reader(response.into_reader()).unwrap()
        })
    }
    pub fn get_series_details_now(&self, id: u32) -> SeriesDetails {
        let url = format!("{SERIES_DETAILS_URL}{id}");
        let request = self.new_authorized_get(&url);

        println!("Executing request in get_series_details_now");

        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        serde_json::from_reader(response.into_reader()).unwrap()
    }

    pub fn get_season_details(&self, series_id: u32, season_number: u32) -> Job<SeasonDetails> {
        let url = format!("{SERIES_DETAILS_URL}{series_id}/season/{season_number}");
        let request = self.new_authorized_get(&url);

        Job::new(move || {
            println!("Executing request in get_season_details");
            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };

            serde_json::from_reader(response.into_reader()).unwrap()
        })
    }

    pub fn get_seasons(&self, series_id: u32) {
        let url = format!("{SERIES_DETAILS_URL}{series_id}/season/&append_to_response=3");
        let request = self.new_authorized_get(&url);

        println!("Executing request in get_season_details");
        match request.call() {
            Ok(response) => {
                let json_response = response.into_string().unwrap();
                println!("{}", json_response);
            },
            Err(err) => {
                eprintln!("{}", err.to_string())
            }
        }
    }

    pub fn get_movie_details(&self, movie_id: u32) -> Job<MovieDetails> {
        let url = format!("{MOVIE_DETAILS_URL}{movie_id}");
        let request = self.new_authorized_get(&url);

        Job::new(move || {
            println!("Executing request in get_movie_details");
            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };

            let json_response = response.into_string().unwrap();
            let mut payload: Value = serde_json::from_str(json_response.as_str()).unwrap();
            let mut taken_genres = payload["genres"].take();
            let genres_arr = taken_genres.as_array_mut().unwrap();
            let mut details: MovieDetails = serde_json::from_value(payload).unwrap();
            let mut genres: Vec<String> = Vec::with_capacity(genres_arr.len());
            for genre_obj in genres_arr {
                genres.push(genre_obj["name"].take().to_string());
            }
            details.genres = genres;
            details
        })
    }

    fn get_imdb_url(&self, endpoint_url: String, prod_name: &str) -> String {
        let request = self.new_authorized_get(&endpoint_url);
        let Ok(response) = request.call() else {
            eprintln!("Error on sending request");
            return format!("{IMDB_FIND}{prod_name}");
        };
        let mut json: Value = serde_json::from_reader(response.into_reader()).unwrap();
        println!("{}", json);
        let ids: ProductionIds = serde_json::from_value(json.take()).unwrap();
        match ids.imdb_id {
            Some(imdb_id) => format!("{IMDB_TITLE}{imdb_id}"),
            None => format!("{IMDB_FIND}{prod_name}"),
        }
    }

    pub fn get_imdb_url_movie(&self, title: &str, movie_id: u32) -> String {
        let url = format!("{MOVIE_DETAILS_URL}/{movie_id}/external_ids");
        self.get_imdb_url(url, title)
    }
    pub fn get_imdb_url_series(&self, name: &str, series_id: u32) -> String {
        let url = format!("{SERIES_DETAILS_URL}/{series_id}/external_ids");
        self.get_imdb_url(url, name)
    }

    pub fn get_movie_trailers(&self, movie_id: u32) -> Vec<Trailer> {
        self.get_trailers(format!("https://api.themoviedb.org/3/movie/{movie_id}/videos"))
    }

    pub fn get_series_trailers(&self, series_id: u32) -> Vec<Trailer> {
        self.get_trailers(format!("https://api.themoviedb.org/3/tv/{series_id}/videos"))
    }

    fn get_trailers(&self, url: String) -> Vec<Trailer> {
        let request = self.new_authorized_get(&url);
        let Ok(response) = request.call() else {
            eprintln!("Error on sending request");
            return Vec::new();
        };
        let mut json: Value = serde_json::from_reader(response.into_reader()).unwrap();
        let mut results_arr = json["results"].take();
        let videos = results_arr.as_array_mut().unwrap();
        let mut trailers: Vec<Trailer> = Vec::new();
        for vid in videos {
            if vid["type"] == "Trailer" {
                let trailer = serde_json::from_value(vid.take()).unwrap();
                trailers.push(trailer);
            }
        }
        trailers
    }

    fn get_keywords(&self, url: String, array_key: &str) -> Vec<Keyword> {
        let request = self.new_authorized_get(&url);
        let Ok(response) = request.call() else {
            eprintln!("Error on sending request");
            return Vec::new();
        };

        let mut json: Value = serde_json::from_reader(response.into_reader()).unwrap();
        let mut keywords_arr = json[array_key].take();
        let json_keywords = keywords_arr.as_array_mut().unwrap();
        let mut keywords: Vec<Keyword> = Vec::with_capacity(json_keywords.len());
        for keyword in json_keywords {
            keywords.push(serde_json::from_value(keyword.take()).unwrap());
        }
        keywords
    }

    pub fn get_keywords_movie(&self, movie_id: u32) -> Vec<Keyword> {
        let url = format!("{MOVIE_DETAILS_URL}/{movie_id}/keywords");
        self.get_keywords(url, "keywords")
    }
    pub fn get_keywords_series(&self, series_id: u32) -> Vec<Keyword> {
        let url = format!("{SERIES_DETAILS_URL}/{series_id}/keywords");
        self.get_keywords(url, "results")
    }

    pub fn download_poster(&self, poster_url: &str, file_path: &str) {
        let agent = self.agent.clone();
        let mut file = std::fs::File::create(file_path).expect("looks like someone tried to unwrap...");
        let request = agent.get(poster_url);

        std::thread::spawn(move || {
            println!("Executing request in download_poster");
            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };
            let bytes_written = std::io::copy(&mut response.into_reader(), &mut file).unwrap();
            println!("bytes written {}", bytes_written);
        });
    }
}

#[allow(dead_code)]
struct VecMap<K, V> {
    keys_to_values: Vec<(K, V)>,
}

#[allow(dead_code)]
impl<K: PartialEq, V> VecMap<K, V> {
    pub fn new() -> VecMap<K, V> {
        Self { keys_to_values: vec![] }
    }

    pub fn put_value(&mut self, key: K, value: V) {
        self.keys_to_values.push((key, value));
    }

    // pub fn put_shared(&mut self, key: K, value: Arc<V>){
    //     self.keys_to_values.push((key, value));
    // }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys_to_values.iter().find(|(k, _)| k.eq(key)).map(|(_, v)| v)
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
    pub fn size(&self) -> usize {
        self.keys_to_values.len()
    }
}
