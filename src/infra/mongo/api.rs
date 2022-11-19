use mongodb::{bson::doc, sync::Client};
use std::error::Error;

pub struct MongoRep {
    pub ingredients: mongodb::Collection<Ingredient>,
    pub recipes: mongodb::Collection<Recipe>,
}

fn connect(uri: String) {}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the MongoDB connection string from an environment variable:
    // Careful ! You need to export your connection string here!

    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;

    // Print the databases in our MongoDB cluster:
    println!("Databases:");
    for name in client.list_database_names(None, None).await? {
        println!("- {}", name);
    }

    Ok(())
}
