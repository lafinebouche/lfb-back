mod infra;

use infra::*;
use rocket::{http::Status, serde::json::Json, State};

#[macro_use]
extern crate rocket;

#[get("/ingredient/<id>")]
fn get_ingredient(db: &State<MongoRep>, id: String) -> Result<Json<Ingredient>, Status> {
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let result = db.get_ingredient(id);

    match result {
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

#[launch]
fn rocket() -> _ {
    let db = MongoRep::init(String::from(""), "");
    rocket::build().manage(db)
    // .mount("/", routes![get_ingredient, get_recipes])
}
