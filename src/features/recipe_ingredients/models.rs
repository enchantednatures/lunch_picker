use serde::Serialize;

use crate::features::ingredients::{IngredientId, IngredientName, Measure};

#[derive(Debug, PartialEq, Serialize)]
pub struct RecipeIngredient {
    pub recipe_id: i32,
    pub ingredient_id: IngredientId,
    pub ingredient_name: IngredientName,
    pub quantity: f64,
    pub measure: Measure,
    pub raw_ingredient: Option<String>,
}
