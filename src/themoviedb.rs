use crate::config::Config;
use crate::production::{Movie, Production, Series};
use crate::series_details::{SeasonDetails, SeriesDetails};
use ureq;
use serde_json::Value;

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
    pub config: Config,
    // client: Client,
}

impl TheMovieDB {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            // client: Client::new(),
        }
    }

    fn new_authorized_get(&self, url: String) -> ureq::Request {
        let request = ureq::get(&url)
            .set("Accept", "application/json")
            .set("Authorization", &format!("Bearer {}", &self.config.api_key));
        request
    }

    pub fn search_production(&self, query: &str) -> Vec<Production> {
        let mut url = String::from(SEARCH_MULTI_URL);
        url.push_str(format!("?query={}&include_adult={}", query, true).as_str());

        let request = self.new_authorized_get(url);

        println!("Executing request..");
        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let status = response.status();
        if status != 200 {
            println!("status: {}", status)
        }

        let json_response = response.into_string().unwrap().to_owned();
        println!("content: {}", json_response);

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
                println!("{:?}", series);
                productions.push(Production::Series(series));
            } else if media_type == "movie" {
                let movie = Movie::parse(prod_obj.to_string().as_str());
                println!("{:?}", movie);
                productions.push(Production::Movie(movie));
            }
        }
        productions
    }

    pub fn get_full_poster_url(poster: &str, width: Width) -> String {
        let mut url = String::from(IMAGE_URL);
        let size = match width {
            Width::W200 => "w200",
            Width::W300 => "w300",
            Width::W400 => "w400",
            Width::W500 => "w500",
            Width::ORIGINAL => "original",
        };
        url.push_str(size);
        url.push_str(poster);
        url
    }

    pub fn get_series_details(&self, id: u32) -> SeriesDetails {
        let mut url = String::from(SERIES_DETAILS_URL);
        url.push_str(id.to_string().as_str());

        let request = self.new_authorized_get(url);
        println!("Executing request..");

        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let json = response.into_string().unwrap();
        println!("series_details_json: {}", json);
        SeriesDetails::parse(json.as_str())
    }

    pub fn get_season_details(&self, series_id: u32, season_number: u32) -> SeasonDetails {
        let mut url = String::from(SERIES_DETAILS_URL);
        url.push_str(series_id.to_string().as_str());
        url.push_str("/season/");
        url.push_str(season_number.to_string().as_str());
        let request = self.new_authorized_get(url);

        println!("Executing request..");
        let Ok(response) = request.call() else {
            panic!("Error on sending request");
        };

        let json = response.into_string().unwrap();
        println!("season_details_json: {}", json);
        SeasonDetails::parse(json.as_str())
    }

    pub fn download_resource(&self, resource_url: &str) -> Vec<u8> {
        let request = ureq::get(resource_url);
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
