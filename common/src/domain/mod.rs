use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Homie {
    pub id: i64,
    pub name: String,
}

impl Homie {
    pub fn new(id: i64, name: String) -> Self {
        Self { id, name }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
}

impl Recipe {
    pub fn new(id: i64, name: String) -> Self {
        Self { id, name }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HomiesFavorite {
    pub homie_id: i64,
    pub recipe_id: i64,
}

impl HomiesFavorite {
    pub fn new(homie_id: i64, recipe_id: i64) -> Self {
        Self {
            homie_id,
            recipe_id,
        }
    }
}
