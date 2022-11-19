use super::types::{Ingredient, Recipe};
use mongodb::{bson::doc, error::Error as mongoError, sync::Client};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MongoRepError {
    #[error("error querying value")]
    QueryError(#[from] mongoError),
    #[error("missing ingredient {0}")]
    InvalidIngredientName(String),
}

pub struct MongoRep {
    pub ingredients: mongodb::sync::Collection<Ingredient>,
    pub recipes: mongodb::sync::Collection<Recipe>,
}

impl MongoRep {
    fn init(uri: String, data: &str) -> Result<Self, Box<dyn Error>> {
        let client = Client::with_uri_str(uri)?;
        let database = client.database(data);
        let rep = MongoRep {
            ingredients: database.collection("ingredients"),
            recipes: database.collection("recipes"),
        };
        return Ok(rep);
    }

    fn get_ingredient(self, name: String) -> Result<Ingredient, MongoRepError> {
        match self
            .ingredients
            .find_one(doc! {"name": &name}, None)
            .map_err(MongoRepError::from)?
        {
            Some(ing) => Ok(ing),
            _ => Err(MongoRepError::InvalidIngredientName(name)),
        }
    }
}
