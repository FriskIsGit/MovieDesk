use reqwest::blocking::{Client, Request, Response, RequestBuilder};
use serde_json::json;
use crate::config::Config;

const SEARCH_MOVIE_URL: &str = "https://api.themoviedb.org/3/search/movie";

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

    pub fn search_movie(&self, query: &str) -> Vec<Movie>{
        let mut url = String::from(SEARCH_MOVIE_URL);
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
        println!("content: {}", response.text().unwrap());
        return Vec::new();
    }
}

pub struct Movie{

}

impl Movie{

}
