use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Ingredient {
    pub domain: String,
    // TODO change from string to hex string
    pub hash: String,
    // TODO change from string to hex string
    // pub path: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Recipe {
    pub ingredients: Vec<Ingredient>,
}
