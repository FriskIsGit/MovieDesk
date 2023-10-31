use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credits {
    id: u32,
    //simplify it to cast at some point
    cast: Vec<Actor>,
    //don't deserialize crew
}

impl Credits {
    pub fn parse(json: &str) -> Credits {
        serde_json::from_str(json).expect("Failed to deserialize a Credits object")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor {
    character: String,
    name: String,
    original_name: String,
    credit_id: u32,
    gender: u32,
    adult: bool,
}

impl Actor {
    pub fn get_gender(&self) -> &str {
        match self.gender {
            0 => "Unset",
            1 => "Female",
            2 => "Male",
            3 => "Non-binary",
            _ => "Unreachable",
        }
    }
}
