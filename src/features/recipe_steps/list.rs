use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::RecipeStep;

#[tracing::instrument(skip(db))]
pub async fn list_recipe_steps(
    recipe_id: i32,
    db: &impl ListRecipeSteps,
) -> Result<Vec<RecipeStep>, ListRecipeStepsError> {
    let steps = db.list_recipe_steps(recipe_id).await?;
    Ok(steps)
}

#[derive(Error, Debug)]
pub enum ListRecipeStepsError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListRecipeSteps {
    async fn list_recipe_steps(
        &self,
        recipe_id: i32,
    ) -> Result<Vec<RecipeStep>, ListRecipeStepsError>;
}

impl ListRecipeSteps for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_recipe_steps(
        &self,
        recipe_id: i32,
    ) -> Result<Vec<RecipeStep>, ListRecipeStepsError> {
        let rows = sqlx::query_as::<_, RecipeStep>(
            r#"
            SELECT id, recipe_id, step_number, instruction
            FROM recipe_steps
            WHERE recipe_id = ?
            ORDER BY step_number
            "#,
        )
        .bind(recipe_id)
        .fetch_all(self)
        .instrument(tracing::info_span!("List Recipe Steps"))
        .await
        .map_err(|e| ListRecipeStepsError::UnknownDbError(e))?;

        Ok(rows)
    }
}
