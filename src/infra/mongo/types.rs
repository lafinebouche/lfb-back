use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Ingredient {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub domain: String,
    // TODO change from string to hex string
    pub hash: String,
    // TODO change from string to hex string
    // TODO add merkle tree path
    // pub path: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Recipe {
    // TODO change from string to hex string
    pub address: String,
    pub status: Status,
    pub ingredients: Vec<DbIngredient>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DbIngredient {
    pub id: ObjectId,
    pub status: Status,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Status {
    Ongoing,
    Completed,
}
