use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    pub title: String,
    pub original_language: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: String,
    pub vote_average: f32,
    pub vote_count: u32,
    pub adult: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieDetails {
    pub backdrop_path: Option<String>,
    pub budget: usize,
    pub revenue: usize,
    pub runtime: u32,
    pub imdb_id: Option<String>,
    pub original_title: Option<String>,
    pub production_companies: Vec<ProductionCompany>,
    pub status: String,
    pub tagline: String,

    #[serde(skip_deserializing)]
    pub genres: Vec<String>,
}

impl MovieDetails {
    // always provided in USD?
    pub fn revenue(&self) -> String {
        self.format(self.revenue)
    }
    pub fn budget(&self) -> String {
        self.format(self.budget)
    }
    fn format(&self, amount: usize) -> String {
        let mut format = amount.to_string();
        let len = format.len();
        if len < 5 {
            format.push('$');
            return format;
        }
        let mut st = len % 3;
        if st == 0 {
            st = 3;
        }
        for i in (st..format.len()).step_by(4) {
            if i == format.len() {
                break;
            }
            format.insert(i, ' ');
        }
        format.push('$');
        format
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionCompany {
    pub id: u32,
    pub name: String,
    pub origin_country: String,
    pub logo_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserMovie {
    pub movie: Movie,
    pub user_rating: f32,
    pub note: String,
}

impl UserMovie {
    pub fn new(movie: Movie) -> Self {
        Self {
            movie,
            note: String::new(),
            user_rating: 0.0,
        }
    }
}
