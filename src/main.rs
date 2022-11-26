mod infra;

use infra::*;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    let db = MongoRep::init("mongodb://localhost:27017/".to_string(), "lfb").unwrap();
    rocket::build()
        .manage(db)
        .mount("/", routes![get_ingredient])
}
