use super::types::{DbIngredient, Ingredient, Recipe};
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Error as mongoError,
    sync::Client,
};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MongoRepError {
    #[error("error querying value")]
    QueryError(#[from] mongoError),
    #[error("missing ingredient {0}")]
    InvalidIngredientName(String),
    #[error("recipe not found for ingredients")]
    InvalidIngredientsList(),
    #[error("incorrect ingredients list {0}, expected between 2 and 6 ingredients")]
    IncorrectIngredientsLength(usize),
    #[error("empty response")]
    EmptyResponse(),
}

pub struct MongoRep {
    pub ingredients: mongodb::sync::Collection<Ingredient>,
    pub recipes: mongodb::sync::Collection<Recipe>,
}

impl MongoRep {
    pub fn init(uri: String, database: &str) -> Result<Self, MongoRepError> {
        let client = Client::with_uri_str(uri)?;
        let database = client.database(database);
        let rep = MongoRep {
            ingredients: database.collection("ingredients"),
            recipes: database.collection("recipes"),
        };
        return Ok(rep);
    }

    pub fn get_ingredient(&self, name: &str) -> Result<Ingredient, MongoRepError> {
        match self
            .ingredients
            .find_one(doc! {"domain": &name}, None)
            .map_err(MongoRepError::from)?
        {
            Some(ing) => Ok(ing),
            _ => Err(MongoRepError::InvalidIngredientName(String::from(name))),
        }
    }

    pub fn get_ingredients(
        &self,
        ingredients: Vec<&str>,
    ) -> Result<Vec<Ingredient>, MongoRepError> {
        let cursor = self
            .ingredients
            .find(doc! {"domain": {"$in": ingredients}}, None)
            .map_err(MongoRepError::from)?;
        match cursor.collect::<Result<Vec<Ingredient>, mongoError>>() {
            Ok(v) if v.len() > 0 => Ok(v),
            Ok(_) => Err(MongoRepError::EmptyResponse()),
            _ => Err(MongoRepError::InvalidIngredientsList()),
        }
    }

    pub fn get_recipe(&self, ingredients: Vec<&str>) -> Result<Vec<Recipe>, MongoRepError> {
        let len = ingredients.len();
        if len > 6 || len < 2 {
            return Err(MongoRepError::IncorrectIngredientsLength(len));
        }
        let ingredients = self.get_ingredients(ingredients)?;
        let ids: Vec<mongodb::bson::Document> = ingredients
            .into_iter()
            //TODO improve the handling of None
            .map(|x| x.id.unwrap_or_else(|| ObjectId::new()))
            .map(|x| doc! { "$elemMatch": {"id": x} })
            .collect();
        let cursor = self
            .recipes
            .find(doc! {"ingredients": {"$all": ids}}, None)
            .map_err(MongoRepError::from)?;
        match cursor.collect::<Result<Vec<Recipe>, mongoError>>() {
            Ok(v) if v.len() > 0 => Ok(v),
            Ok(_) => Err(MongoRepError::EmptyResponse()),
            Err(e) => Err(MongoRepError::InvalidIngredientsList()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::infra::mongo::types::Status;

    use super::*;

    fn init_repo(database: &str) -> MongoRep {
        MongoRep::init(String::from("mongodb://localhost:27017/"), database).unwrap()
    }

    #[test]
    fn test_init_mongo_repo_passes() {
        init_repo("test");
    }

    #[test]
    #[should_panic(expected = "InvalidIngredientName")]
    fn test_get_ingredient_invalid_ingredient_query() {
        let mongo_rep = init_repo("lfb");
        mongo_rep.get_ingredient("hello.eth").unwrap();
    }

    #[test]
    fn test_get_ingredient_passes() {
        let mongo_rep = init_repo("lfb");
        let ingredient = mongo_rep.get_ingredient("abricot.eth").unwrap();
        assert_eq!("abricot.eth", ingredient.domain);
        assert_eq!(
            mongodb::bson::oid::ObjectId::from_str("637be8b4942c929a6d8710c9").unwrap(),
            ingredient.id.unwrap()
        )
    }

    #[test]
    #[should_panic(expected = "EmptyResponse")]
    fn test_get_ingredients_invalid_ingredient_query() {
        let mongo_rep = init_repo("lfb");
        mongo_rep.get_ingredients(vec!["hello.eth"]).unwrap();
    }

    #[test]
    fn test_get_ingredients_passes() {
        let mongo_rep = init_repo("lfb");
        let ingredients = mongo_rep
            .get_ingredients(vec!["abricot.eth", "ail.eth"])
            .unwrap();
        assert_eq!(ingredients[0].domain, "abricot.eth");
        assert_eq!(ingredients[1].domain, "ail.eth");
    }

    #[test]
    #[should_panic(expected = "IncorrectIngredientsLength")]
    fn test_get_recipe_incorrect_ingredients_list_length() {
        let mongo_rep = init_repo("lfb");
        mongo_rep.get_recipe(vec!["hello.eth"]).unwrap();
    }

    #[test]
    #[should_panic(expected = "EmptyResponse")]
    fn test_get_recipe_invalid_ingredients_query() {
        let mongo_rep = init_repo("lfb");
        mongo_rep
            .get_recipe(vec!["hello.eth", "there.eth"])
            .unwrap();
    }

    #[test]
    fn test_get_recipe_passes() {
        let mongo_rep = init_repo("lfb");
        let recipe = mongo_rep
            .get_recipe(vec!["agaragar.eth", "asperge.eth"])
            .unwrap();
        assert_eq!(recipe[0].address, "0x12345");
        assert_eq!(recipe[0].status, Status::Ongoing);
    }
}
