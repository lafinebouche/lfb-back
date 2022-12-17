mod infra;
use infra::*;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

#[macro_use]
extern crate rocket;
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    let db = MongoRep::init("mongodb://localhost:27017/".to_string(), "lfb").unwrap();
    rocket::build()
        .manage(db)
        .mount(
            "/",
            routes![
                get_ingredient,
                get_recipes,
                get_ingredients_by_id,
                get_leaderboard,
                get_statistics
            ],
        )
        .attach(CORS)
}
