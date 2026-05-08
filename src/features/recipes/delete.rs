use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn delete_recipe(
    recipe_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl DeleteRecipe,
) -> Result<(), DeleteRecipeError> {
    db.delete_recipe(recipe_id, user_id.into()).await?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum DeleteRecipeError {
    #[error("Recipe not found")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait DeleteRecipe {
    async fn delete_recipe(
        &self,
        recipe_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteRecipeError>;
}

impl DeleteRecipe for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn delete_recipe(
        &self,
        recipe_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteRecipeError> {
        let result = sqlx::query(
            r#"DELETE FROM recipes WHERE id = ? AND user_id = ?"#,
        )
        .bind(recipe_id)
        .bind(user_id.as_i32())
        .execute(self)
        .instrument(tracing::info_span!("Delete Recipe"))
        .await
        .map_err(|e| DeleteRecipeError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(DeleteRecipeError::NotFound);
        }

        Ok(())
    }
}
