use serde::Serialize;
use sqlx::FromRow;

use crate::features::ingredients::{IngredientId, IngredientName, Measure};

#[derive(Debug, PartialEq, Serialize)]
pub struct PantryIngredient {
    pub ingredient_id: IngredientId,
    pub ingredient_name: IngredientName,
    pub quantity: f64,
    pub measure: Measure,
}

#[derive(Debug, PartialEq, FromRow)]
pub struct PantryIngredientRow {
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub quantity: f64,
    pub measure: String,
}

impl From<PantryIngredientRow> for PantryIngredient {
    fn from(row: PantryIngredientRow) -> Self {
        Self {
            ingredient_id: IngredientId::from(row.ingredient_id),
            ingredient_name: IngredientName::try_from(row.ingredient_name)
                .unwrap_or_else(|_| IngredientName::try_from("unknown".to_string()).unwrap()),
            quantity: row.quantity,
            measure: Measure::try_from(row.measure.as_str()).unwrap_or(Measure::Each),
        }
    }
}
