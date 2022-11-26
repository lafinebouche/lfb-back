use rocket::{http::Status, serde::json::Json, State};

use super::{Ingredient, MongoRep};

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

// #[get("/recipes/<ids>")]
// fn get_recipes(db: &State<MongodRepo>, ids: Vec<String>) -> Result<Vec<Json<Recipe>>, Status> {
//     let result = db.get_recipes(ingredients);

//     match result {
//         Ok(recipes) => Ok(Json(recipes)),
//         Err(_) => Err(Status::InternalServerError),
//     }
// }
