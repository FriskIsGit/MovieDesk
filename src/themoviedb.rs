use hyper::{Body, Client, Method, Request, Response, Uri};
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::http::request::Builder;
use serde_json::json;
use crate::config::Config;

const SEARCH_MOVIE_URL: &str = "https://api.themoviedb.org/3/search/movie";

pub struct TheMovieDB{
    config: Config,
    client: Client<HttpConnector>
}

impl TheMovieDB{
    pub fn new(config: Config) -> Self{
        Self{
            config,
            client: Client::new()
        }
    }
    fn new_authorized_get(&self) -> Builder {
        Request::builder()
            .method(Method::GET)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.api_key.clone()))
    }

    pub async fn search_movie(&self, query: &str) -> Vec<Movie>{
        println!("exeucting serahc move");
        let mut url = String::from(SEARCH_MOVIE_URL);
        url.push_str(format!("?query={}&include_adult={}", query, true).as_str());
        let request: Request<Body> = self.new_authorized_get()
            .uri(url)
            .body(Body::empty())
            .expect("Should have been a valid request");
        let future = self.client.request(request);
        println!("After future before send");
        let response = future.await.expect("Expected a response");
        println!("Response code: {}", response.status().as_u16());
        let (_, body) = response.into_parts();
        let str_body: String = TheMovieDB::body_to_string(body).await;
        println!("CONTENT: {}", str_body);
        return Vec::new();
    }

    async fn body_to_string(body: Body) -> String {
        if let Ok(bytes) = hyper::body::to_bytes(body).await {
            return String::from_utf8(bytes.to_vec()).unwrap()
        }
        return String::new();
    }
}

pub struct Movie{

}

impl Movie{

}
