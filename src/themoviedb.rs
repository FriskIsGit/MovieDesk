use hyper::{Body, Client, Method, Request, Uri};
use hyper::client::HttpConnector;
use hyper::http::request::Builder;

const SEARCH_MOVIE_URL: &str = "https://api.themoviedb.org/3/search/movie";

struct TheMovieDB{
    api_key: String,
    client: Client<HttpConnector>
}

impl TheMovieDB{
    pub fn new(api_key: &str) -> Self{
        Self{
            api_key: String::from(api_key),
            client: Client::new()
        }
    }
    fn new_authorized_get(&self) -> Builder {
        Request::builder()
            .method(Method::GET)
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key.clone()))
    }

    pub fn search_movie(&self, query: &str) -> Builder{
        let mut url = String::from(SEARCH_MOVIE_URL);
        url.push_str(&*format!("?query={}&include_adult={}", query, true));
        let request = self.new_authorized_get();
        request.uri(url)
    }
}
