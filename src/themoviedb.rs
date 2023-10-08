use reqwest::blocking::{Client, Request, Response, RequestBuilder};
use serde_json::{Value};
use crate::config::Config;
use crate::production;
use crate::production::{Movie, Production, TVShow};
use crate::production::Production::Series;
use crate::production::Production::Film;

const SEARCH_MULTI_URL: &str = "https://api.themoviedb.org/3/search/multi";
const IMAGE_URL: &str = "https://image.tmdb.org/t/p/";

pub struct TheMovieDB{
    config: Config,
    client: Client,
}

impl TheMovieDB{
    pub fn new(config: Config) -> Self{
        Self{
            config,
            client: Client::new()
        }
    }
    fn new_authorized_get(&self, url: String) -> RequestBuilder {
        self.client.get(url)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key.clone()))
    }

    pub fn search_production(&self, query: &str) -> Vec<Production>{
        let mut url = String::from(SEARCH_MULTI_URL);
        url.push_str(format!("?query={}&include_adult={}", query, true).as_str());

        let request = self.new_authorized_get(url);
        println!("Executing request..");
        let result = request.send();
        if !result.is_ok() {
            panic!("Error on sending request");
        }
        let response: Response = result.unwrap();
        let status = response.status();
        if !status.is_success() {
            println!("status: {}", status)
        }
        let json_response = response.text().unwrap().to_owned();
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
                let show = TVShow::parse(prod_obj.to_string().as_str());
                println!("{:?}", show);
                productions.push(Series(show));
            } else if media_type == "movie" {
                let movie = Movie::parse(prod_obj.to_string().as_str());
                println!("{:?}", movie);
                productions.push(Film(movie));
            }
        }
        return productions;
    }
    pub fn get_full_poster_url(&self, poster: &str, width: Width) -> String {
        let mut url = String::from(IMAGE_URL);
        let size = match width {
            Width::W200 => {"w200"}
            Width::W300 => {"w300"}
            Width::W400 => {"w400"}
            Width::W500 => {"w500"}
            Width::ORIGINAL => {"original"}
        };
        url.push_str(size);
        url.push_str(poster);
        return url;
    }
}

pub enum Width{
    W200, W300, W400, W500, ORIGINAL
}