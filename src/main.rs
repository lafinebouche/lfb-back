mod infra;
use std::net::Ipv4Addr;

use infra::*;

use rocket::config::{CipherSuite, TlsConfig};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Config, Request, Response};

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
    let db = MongoRep::init(
        dotenv::var("MONGO_URI").expect("MONGO_URI must be set"),
        "lfb",
    )
    .unwrap();
    let mut config = Config::debug_default();
    config.address = Ipv4Addr::new(0, 0, 0, 0).into();
    config.port = 8000;
    let tls_config = TlsConfig::from_paths("./fullchain1.pem", "privkey1.pem")
        .with_ciphers(CipherSuite::TLS_V13_SET)
        .with_preferred_server_cipher_order(true);
    config.tls = Some(tls_config);
    rocket::build()
        .manage(db)
        .configure(&config)
        .mount(
            "/",
            routes![
                get_ingredient,
                get_recipes,
                get_ingredients_by_id,
                get_leaderboard,
                get_statistics,
                get_ongoing_recipes
            ],
        )
        .attach(CORS)
}
