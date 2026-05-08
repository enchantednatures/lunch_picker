use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, PartialEq, Eq, Serialize, FromRow)]
pub struct RecipeStep {
    pub id: i32,
    pub recipe_id: i32,
    pub step_number: i32,
    pub instruction: String,
}
