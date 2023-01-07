use super::{Ingredient, MongoRep, MongoRepError, Recipe};
use rocket::get;
use rocket::{http::Status, serde::json::Json, State};

#[get("/ingredient/<name>")]
pub fn get_ingredient(db: &State<MongoRep>, name: &str) -> Result<Json<Ingredient>, Status> {
    println!("{}", name);
    if name.is_empty() {
        return Err(Status::BadRequest);
    };
    let ingredient = db.get_ingredient(&name);

    match ingredient {
        Ok(ingredient) => Ok(Json(ingredient)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/statistics/<addr>")]
pub fn get_statistics(db: &State<MongoRep>, addr: &str) -> Result<Json<Vec<(u32, u32)>>, Status> {
    let stats = db.get_statistics(addr);

    match stats {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/leaderboard")]
pub fn get_leaderboard(db: &State<MongoRep>) -> Result<Json<Vec<(String, u32)>>, Status> {
    let leaderboard = db.get_leaderboard();

    match leaderboard {
        Ok(leaderboard) => Ok(Json(leaderboard)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/ingredients/<ids>")]
pub fn get_ingredients_by_id(
    db: &State<MongoRep>,
    ids: &str,
) -> Result<Json<Vec<Ingredient>>, Status> {
    let ids = ids.split(',').collect();
    let result = db.get_ingredients_by_id(ids);

    match result {
        Ok(ingredients) => Ok(Json(ingredients)),
        Err(MongoRepError::IncorrectIngredientsLength(_)) => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/ongoing-recipes")]
pub fn get_ongoing_recipes(db: &State<MongoRep>) -> Result<Json<Vec<Recipe>>, Status> {
    let result = db.get_recipes_ongoing();
    match result {
        Ok(recipes) => Ok(Json(recipes)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/recipes/<names>")]
pub fn get_recipes(db: &State<MongoRep>, names: &str) -> Result<Json<Vec<Recipe>>, Status> {
    let names = names.split(',').collect();
    let result = db.get_recipes(names);

    match result {
        Ok(recipes) => Ok(Json(recipes)),
        Err(MongoRepError::IncorrectIngredientsLength(_)) => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}
