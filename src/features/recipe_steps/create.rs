use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::RecipeStep;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn create_recipe_step(
    recipe_id: i32,
    step_number: i32,
    instruction: String,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateRecipeStep,
) -> Result<RecipeStep, CreateRecipeStepError> {
    let params = CreateRecipeStepParams::new(recipe_id, step_number, instruction, user_id.into());
    let step = db.create_recipe_step(params).await?;
    Ok(step)
}

#[derive(Debug)]
pub(crate) struct CreateRecipeStepParams {
    recipe_id: i32,
    step_number: i32,
    instruction: String,
    user_id: i32,
}

impl CreateRecipeStepParams {
    fn new(recipe_id: i32, step_number: i32, instruction: String, user_id: UserId) -> Self {
        Self {
            recipe_id,
            step_number,
            instruction,
            user_id: user_id.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateRecipeStepError {
    #[error("Step already exists for this recipe")]
    StepAlreadyExists,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait CreateRecipeStep {
    async fn create_recipe_step(
        &self,
        params: CreateRecipeStepParams,
    ) -> Result<RecipeStep, CreateRecipeStepError>;
}

impl CreateRecipeStep for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_recipe_step(
        &self,
        params: CreateRecipeStepParams,
    ) -> Result<RecipeStep, CreateRecipeStepError> {
        let result = sqlx::query(
            r#"
            INSERT INTO recipe_steps (recipe_id, user_id, step_number, instruction)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(params.recipe_id)
        .bind(params.user_id)
        .bind(params.step_number)
        .bind(&params.instruction)
        .execute(self)
        .instrument(tracing::info_span!("Insert Recipe Step"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateRecipeStepError::StepAlreadyExists;
                }
                CreateRecipeStepError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateRecipeStepError::UnknownDbError(e),
        })?;

        Ok(RecipeStep {
            id: result.last_insert_rowid() as i32,
            recipe_id: params.recipe_id,
            step_number: params.step_number,
            instruction: params.instruction,
        })
    }
}
