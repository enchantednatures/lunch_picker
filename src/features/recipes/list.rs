use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{Recipe, RecipeRow};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn list_recipes(
    user_id: impl Into<UserId> + std::fmt::Debug,
    db: &impl ListRecipes,
) -> Result<Vec<Recipe>, ListRecipesError> {
    let recipes = db.list_recipes(user_id.into()).await?;
    Ok(recipes.into_iter().map(|r| r.into()).collect())
}

#[derive(Error, Debug)]
pub enum ListRecipesError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListRecipes {
    async fn list_recipes(&self, user_id: UserId) -> Result<Vec<RecipeRow>, ListRecipesError>;
}

impl ListRecipes for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_recipes(&self, user_id: UserId) -> Result<Vec<RecipeRow>, ListRecipesError> {
        let rows = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, user_id, name, description, prep_time, cook_time, servings, source_url, imported_at
            FROM recipes
            WHERE user_id = ?
            ORDER BY name
            "#,
        )
        .bind(user_id.as_i32())
        .fetch_all(self)
        .instrument(tracing::info_span!("List Recipes"))
        .await
        .map_err(|e| ListRecipesError::UnknownDbError(e))?;

        Ok(rows)
    }
}
