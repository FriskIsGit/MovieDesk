use crate::jobs::Job;
use crate::production::{Production, ProductionIds, Trailer};
use crate::series_details::{SeasonDetails, SeriesDetails};
use egui::TextBuffer;
use serde_json::Value;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

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

    pub fn search_production(&mut self, query: String) -> Job<(String, Vec<Production>)> {
        let url = format!("{SEARCH_MULTI_URL}?query={}&include_adult={}", query, true);
        let request = self.new_authorized_get(&url);

        Job::new(move || {
            println!("Executing request..");
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
                return (query, vec![]);
            }
            let list = arr.as_array_mut().unwrap();
            let mut productions = Vec::with_capacity(list.len());

            for prod_obj in list {
                let media_type = &prod_obj["media_type"];
                if media_type == "tv" {
                    let series = serde_json::from_value(prod_obj.take()).unwrap();
                    productions.push(Production::Series(series));
                } else if media_type == "movie" {
                    let movie = serde_json::from_value(prod_obj.take()).unwrap();
                    productions.push(Production::Movie(movie));
                }
            }

            (query, productions)
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
            println!("Executing request..");

            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };

            serde_json::from_reader(response.into_reader()).unwrap()
        })
    }

    pub fn get_season_details(&self, series_id: u32, season_number: u32) -> Job<SeasonDetails> {
        let url = format!("{SERIES_DETAILS_URL}{series_id}/season/{season_number}");
        let request = self.new_authorized_get(&url);

        Job::new(move || {
            println!("Executing request..");
            let Ok(response) = request.call() else {
                panic!("Error on sending request");
            };

            serde_json::from_reader(response.into_reader()).unwrap()
        })
    }

    pub fn get_imdb_url(&self, production: Production) -> String {
        let prod_name;
        let url = match production {
            Production::Movie(movie) => {
                prod_name = movie.title.to_owned();
                format!("{MOVIE_DETAILS_URL}/{}/external_ids", movie.id)
            }
            Production::Series(series) => {
                prod_name = series.name.to_owned();
                format!("{SERIES_DETAILS_URL}/{}/external_ids", series.id)
            }
        };
        let request = self.new_authorized_get(&url);
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

    pub fn download_poster(&self, poster_url: &str, file_path: &str) {
        let agent = self.agent.clone();
        let mut file = std::fs::File::create(file_path).expect("looks like someone tried to unwrap...");
        let request = agent.get(poster_url);

        std::thread::spawn(move || {
            println!("Executing request..");
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
