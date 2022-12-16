use super::types::{Ingredient, Recipe, Status};
use mongodb::{
    bson::{doc, oid::ObjectId},
    error::Error as mongoError,
    options::{FindOptions, UpdateOptions},
    sync::Client,
};
use std::str::FromStr;
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
    #[error("could not add ingredient {0} to database")]
    InvalidAddIngredient(String),
    #[error("could not find ingredient from database")]
    InvalidIngredientHash(),
    #[error("could not add recipe to database")]
    InvalidAddRecipe(),
    #[error("invalid update for recipe at {0}")]
    InvalidUpdate(String),
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

    pub fn get_ingredients_by_hash(
        &self,
        hashes: Vec<&str>,
    ) -> Result<Vec<Ingredient>, MongoRepError> {
        let cursor = self
            .ingredients
            .find(doc! {"hash": {"$in" : hashes}}, None)
            .map_err(MongoRepError::from)?;
        match cursor.collect::<Result<Vec<Ingredient>, mongoError>>() {
            Ok(v) => Ok(v),
            Err(e) => Err(MongoRepError::InvalidIngredientHash()),
        }
    }

    pub fn get_ingredients_by_id(&self, ids: Vec<&str>) -> Result<Vec<Ingredient>, MongoRepError> {
        let ids: Vec<ObjectId> = ids
            .into_iter()
            .map(|x| ObjectId::from_str(x).unwrap())
            .collect::<Vec<ObjectId>>();

        let cursor = self
            .ingredients
            .find(doc! {"_id": {"$in" : ids}}, None)
            .map_err(MongoRepError::from)?;
        match cursor.collect::<Result<Vec<Ingredient>, mongoError>>() {
            Ok(v) => Ok(v),
            Err(e) => Err(MongoRepError::InvalidIngredientHash()),
        }
    }

    pub fn get_recipes(&self, ingredients: Vec<&str>) -> Result<Vec<Recipe>, MongoRepError> {
        let len = ingredients.len();
        if len >= 6 || len < 2 {
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
            Ok(v) => Ok(v),
            Err(e) => Err(MongoRepError::InvalidIngredientsList()),
        }
    }

    pub fn get_recipe(&self, address: &str) -> Result<Recipe, MongoRepError> {
        match self.recipes.find_one(doc! {"address": address}, None) {
            Ok(v) => Ok(v.unwrap()),
            Err(e) => Err(MongoRepError::QueryError(e)),
        }
    }

    pub fn get_recipes_ongoing(&self) -> Result<Vec<Recipe>, MongoRepError> {
        let cursor = self
            .recipes
            .find(doc! {"status": "Ongoing"}, None)
            .map_err(MongoRepError::from)?;
        match cursor.collect::<Result<Vec<Recipe>, mongoError>>() {
            Ok(v) => Ok(v),
            Err(e) => Err(MongoRepError::QueryError(e)),
        }
    }

    pub fn add_recipe(
        &self,
        address: &str,
        hashes: Vec<&str>,
        block: i64,
    ) -> Result<bool, MongoRepError> {
        let ingredients = self.get_ingredients_by_hash(hashes).unwrap_or_default();
        let ingredients: Vec<mongodb::bson::Document> = ingredients
            .iter()
            .map(|x| doc! {"id": x.id.unwrap(), "status": "Ongoing", "owner": ""})
            .collect();

        let mut option = UpdateOptions::default();
        option.upsert = Some(true);
        match self
            .recipes
            .update_one(
                doc! {"address": address.to_string()},
                doc! {"$setOnInsert": {"address": address.to_string(), "status": "Ongoing", "ingredients": ingredients, "last_block": block}},
                option,
            )
            .map_err(MongoRepError::from)
        {
            Ok(_) => Ok(true),
            Err(_) => Err(MongoRepError::InvalidAddRecipe()),
        }
    }

    pub fn update_recipe(
        &self,
        address: &str,
        hash: &str,
        owner: &str,
        block: i64,
    ) -> Result<bool, MongoRepError> {
        let ingredient = &self.get_ingredients_by_hash(vec![hash])?[0];
        match self
            .recipes
            .update_one(
                doc! {"address": address.to_string(), "ingredients.id": ingredient.id},
                doc! {"$set": {"last_block": block, "ingredients.$.status": "Completed", "ingredients.$.owner": owner}},
                None,
            )
            .map_err(MongoRepError::from)
        {
            Ok(_) => Ok(true),
            Err(_) => Err(MongoRepError::InvalidUpdate(address.to_string())),
        }
    }

    pub fn update_recipe_completed(&self, address: &str) -> Result<bool, MongoRepError> {
        let recipe = self.get_recipe(address)?;
        let completed = recipe
            .ingredients
            .into_iter()
            .map(|x| x.status == Status::Completed)
            .fold(true, |acc, x| acc && x);
        if !completed {
            return Ok(false);
        }
        match self
            .recipes
            .update_one(
                doc! {"address": address.to_string()},
                doc! {"$set": {"status": "Completed"}},
                None,
            )
            .map_err(MongoRepError::from)
        {
            Ok(_) => Ok(true),
            Err(_) => Err(MongoRepError::InvalidUpdate(address.to_string())),
        }
    }

    pub fn get_leaderboard(&self) -> Result<Vec<(String, u32)>, MongoRepError> {
        let cursor = self.recipes.aggregate(
            vec![
                doc! {"$unwind": "$ingredients"},
                doc! {"$match": {"ingredients.status": "Completed"}},
                doc! {"$group": {
                  "_id": "$ingredients.owner",
                  "count": {
                    "$sum": 1
                  }
                }},
            ],
            None,
        )?;
        match cursor
            .map(|x| {
                let doc = x.unwrap_or_default();
                (
                    doc.get_str("_id").unwrap().to_string(),
                    doc.get_i32("count").unwrap() as u32,
                )
            })
            .collect::<Vec<(String, u32)>>()
        {
            v => Ok(v),
            _ => Err(MongoRepError::EmptyResponse()),
        }
    }

    pub fn get_last_block(&self) -> Result<i64, MongoRepError> {
        let find_options = FindOptions::builder()
            .sort(doc! {"last_block": -1})
            .limit(1)
            .build();
        let cursor = self
            .recipes
            .find(doc! {}, find_options)
            .map_err(MongoRepError::from)?;
        match cursor.collect::<Result<Vec<Recipe>, mongoError>>() {
            Ok(v) if v.len() == 0 => Ok(0),
            Ok(v) => Ok(v.first().unwrap().last_block),
            Err(e) => Err(MongoRepError::QueryError(e)),
        }
    }
}

#[cfg(test)]
mod tests {

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
    fn test_get_last_block_without_data() {
        let mongo_rep = init_repo("lfb");
        let block = mongo_rep.get_last_block().unwrap();
        assert_eq!(block, 0);
    }
    #[test]
    #[should_panic(expected = "InvalidIngredientName")]
    fn test_get_ingredient_invalid_ingredient_query() {
        let mongo_rep = init_repo("lfb");
        mongo_rep.get_ingredient("hello.eth").unwrap();
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
    fn test_get_ingredients_from_hash() {
        let mongo_rep = init_repo("lfb");
        let ingredients: Vec<Ingredient> = mongo_rep
            .get_ingredients_by_hash(vec![
                "0x8574ea6bd913dd9b95296e9e5cede2d361f64f9b4a2f641b5fae3a2948be331e",
                "0xbb46ee301b409e685fdca2667a94deffe378f7081edb25cee0386dc0cd5c2aca",
            ])
            .unwrap();
        // dbg!(ingredients)
        assert_eq!(ingredients[0].domain, "abricot.eth");
        assert_eq!(ingredients[1].domain, "agaragar.eth");
    }

    #[test]
    #[should_panic(expected = "IncorrectIngredientsLength")]
    fn test_get_recipe_incorrect_ingredients_list_length() {
        let mongo_rep = init_repo("lfb");
        mongo_rep.get_recipes(vec!["hello.eth"]).unwrap();
    }

    #[test]
    #[should_panic(expected = "EmptyResponse")]
    fn test_get_recipe_invalid_ingredients_query() {
        let mongo_rep = init_repo("lfb");
        mongo_rep
            .get_recipes(vec!["hello.eth", "there.eth"])
            .unwrap();
    }

    #[test]
    fn test_get_recipe_passes() {
        let mongo_rep = init_repo("lfb");
        assert_eq!(
            "0x1245425523",
            mongo_rep.get_recipe("0x1245425523").unwrap().address
        );
    }

    #[test]
    fn test_get_recipes_passes() {
        let mongo_rep = init_repo("lfb");
        let recipe = mongo_rep
            .get_recipes(vec!["abricot.eth", "ail.eth"])
            .unwrap();
        assert_eq!(recipe[0].address, "0x1245425523");
        assert_eq!(recipe[0].status, Status::Ongoing);
    }

    #[test]
    fn test_get_recipe_ongoing_passes() {
        let mongo_rep = init_repo("lfb");
        let recipe = mongo_rep.get_recipes_ongoing().unwrap();
        assert_eq!("0x1245425523", recipe[0].address);
        assert_eq!(Status::Ongoing, recipe[0].status);
    }

    #[test]
    fn test_add_recipe_passes() {
        let mongo_rep = init_repo("lfb");
        let ingredients = mongo_rep
            .get_ingredients(vec!["abricot.eth", "ail.eth", "aiguillettedecanard.eth"])
            .unwrap();
        assert!(mongo_rep
            .add_recipe(
                "0x1245425523",
                ingredients.iter().map(|x| x.hash.as_str()).collect(),
                1234,
            )
            .unwrap());
    }

    #[test]
    fn test_update_recipe_and_complete_passes() {
        let mongo_rep = init_repo("lfb");
        // update all ingredients of recipe 0x1245425523
        assert!(mongo_rep
            .update_recipe(
                "0x1245425523",
                "0x8574ea6bd913dd9b95296e9e5cede2d361f64f9b4a2f641b5fae3a2948be331e",
                "tim",
                12345,
            )
            .unwrap());
        assert!(mongo_rep
            .update_recipe(
                "0x1245425523",
                "0x3e72143cf7e2a5dd27e9d0ad6bd7f09d98b983f9b05e3e57a07d37b385a9504a",
                "alice",
                12346,
            )
            .unwrap());
        assert!(mongo_rep
            .update_recipe(
                "0x1245425523",
                "0x659ede6c695c50ddd8eb948402f1d4164a77fc60d84b7e473eb9058b40444821",
                "bob",
                12347,
            )
            .unwrap());

        mongo_rep.update_recipe_completed("0x1245425523").unwrap();
        let recipe = mongo_rep.get_recipe("0x1245425523").unwrap();
        assert_eq!(Status::Completed, recipe.status);
    }

    #[test]
    fn test_get_leaderboard() {
        let mongo_rep = init_repo("lfb");
        let leaderboard = mongo_rep.get_leaderboard();
        // TODO update to assert_eq!
        dbg!(leaderboard);
    }

    #[test]
    fn test_last_block_passes() {
        let mongo_rep = init_repo("lfb");
        let ingredients = mongo_rep
            .get_ingredients(vec![
                "abricot.eth",
                "ail.eth",
                "agaragar.eth",
                "aiguillettedecanard.eth",
            ])
            .unwrap();
        assert!(mongo_rep
            .add_recipe(
                "0x1245425524",
                ingredients.iter().map(|x| x.hash.as_str()).collect(),
                12348,
            )
            .unwrap());
        let last_block = mongo_rep.get_last_block().unwrap();
        assert_eq!(last_block, 12348);
    }
}
