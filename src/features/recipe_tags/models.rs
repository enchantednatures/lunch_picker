use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, PartialEq, Eq, Serialize, FromRow)]
pub struct RecipeTag {
    pub recipe_id: i32,
    pub tag_id: i32,
    pub tag_name: String,
}
