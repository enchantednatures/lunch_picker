use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{Recipe, RecipeName, RecipeRow};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn get_recipe_by_name(
    recipe_name: impl AsRef<str> + std::fmt::Debug,
    user_id: impl Into<UserId> + std::fmt::Debug,
    db: &impl GetRecipeByName,
) -> Result<Recipe, GetRecipeError> {
    let recipe = db
        .get_recipe_by_name(recipe_name.as_ref(), user_id.into())
        .await?;
    Ok(recipe)
}

#[derive(Error, Debug)]
pub enum GetRecipeError {
    #[error("Recipe not found")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait GetRecipeByName {
    async fn get_recipe_by_name(
        &self,
        name: &str,
        user_id: UserId,
    ) -> Result<Recipe, GetRecipeError>;
}

impl GetRecipeByName for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn get_recipe_by_name(
        &self,
        name: &str,
        user_id: UserId,
    ) -> Result<Recipe, GetRecipeError> {
        let recipe: RecipeRow = sqlx::query_as(
            r#"
            SELECT id, user_id, name, description, prep_time, cook_time, servings, source_url, imported_at
            FROM recipes
            WHERE user_id = ? AND name = ?
            "#,
        )
        .bind(user_id.as_i32())
        .bind(name)
        .fetch_one(self)
        .instrument(tracing::info_span!("Get Recipe By Name"))
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => GetRecipeError::NotFound,
            _ => GetRecipeError::UnknownDbError(e),
        })?;

        Ok(recipe.into())
    }
}
