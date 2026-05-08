use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn delete_recipe_step(
    step_id: i32,
    user_id: impl Into<UserId> + Debug,
    db: &impl DeleteRecipeStep,
) -> Result<(), DeleteRecipeStepError> {
    db.delete_recipe_step(step_id, user_id.into()).await?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum DeleteRecipeStepError {
    #[error("Step not found")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait DeleteRecipeStep {
    async fn delete_recipe_step(
        &self,
        step_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteRecipeStepError>;
}

impl DeleteRecipeStep for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn delete_recipe_step(
        &self,
        step_id: i32,
        user_id: UserId,
    ) -> Result<(), DeleteRecipeStepError> {
        let result = sqlx::query(
            r#"
            DELETE FROM recipe_steps
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(step_id)
        .bind(user_id.as_i32())
        .execute(self)
        .instrument(tracing::info_span!("Delete Recipe Step"))
        .await
        .map_err(|e| DeleteRecipeStepError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(DeleteRecipeStepError::NotFound);
        }

        Ok(())
    }
}
