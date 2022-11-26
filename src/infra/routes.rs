use rocket::{http::Status, serde::json::Json, State};

use super::{Ingredient, MongoRep, MongoRepError, Recipe};

#[get("/ingredient/<name>")]
pub fn get_ingredient(db: &State<MongoRep>, name: &str) -> Result<Json<Ingredient>, Status> {
    if name.is_empty() {
        return Err(Status::BadRequest);
    };
    let ingredient = db.get_ingredient(&name);

    match ingredient {
        Ok(ingredient) => Ok(Json(ingredient)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/recipes/<names>")]
pub fn get_recipes(db: &State<MongoRep>, names: &str) -> Result<Json<Vec<Recipe>>, Status> {
    println!("{}", names);
    let names = names.split(',').collect();
    let result = db.get_recipes(names);

    match result {
        Ok(recipes) => Ok(Json(recipes)),
        Err(MongoRepError::IncorrectIngredientsLength(_)) => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}
