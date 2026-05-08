use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::features::ingredients::{IngredientName, IngredientNameValidationError};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn remove_recipe_ingredient(
    recipe_id: i32,
    ingredient_name: impl TryInto<IngredientName, Error = IngredientNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl RemoveRecipeIngredient,
) -> Result<(), RemoveRecipeIngredientError> {
    let ingredient_name: IngredientName = ingredient_name.try_into()?;
    let params = RemoveRecipeIngredientParams::new(
        recipe_id,
        ingredient_name.as_str(),
        user_id.into(),
    );

    db.remove_recipe_ingredient(params).await?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct RemoveRecipeIngredientParams<'a> {
    recipe_id: i32,
    ingredient_name: &'a str,
    user_id: i32,
}

impl<'a> RemoveRecipeIngredientParams<'a> {
    fn new(recipe_id: i32, ingredient_name: &'a str, user_id: UserId) -> Self {
        Self {
            recipe_id,
            ingredient_name,
            user_id: user_id.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum RemoveRecipeIngredientError {
    #[error(transparent)]
    ValidationError(#[from] IngredientNameValidationError),

    #[error("Ingredient not found for this recipe")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait RemoveRecipeIngredient {
    async fn remove_recipe_ingredient(
        &self,
        params: RemoveRecipeIngredientParams<'_>,
    ) -> Result<(), RemoveRecipeIngredientError>;
}

impl RemoveRecipeIngredient for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn remove_recipe_ingredient(
        &self,
        params: RemoveRecipeIngredientParams<'_>,
    ) -> Result<(), RemoveRecipeIngredientError> {
        let result = sqlx::query(
            r#"
            DELETE FROM recipe_ingredients
            WHERE recipe_id = ?
              AND ingredient_id = (SELECT id FROM ingredients WHERE name = ?)
            "#,
        )
        .bind(params.recipe_id)
        .bind(params.ingredient_name)
        .execute(self)
        .instrument(tracing::info_span!("Remove Recipe Ingredient"))
        .await
        .map_err(|e| RemoveRecipeIngredientError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(RemoveRecipeIngredientError::NotFound);
        }

        Ok(())
    }
}
