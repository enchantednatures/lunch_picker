use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::RecipeIngredient;
use crate::features::ingredients::{IngredientId, IngredientName, IngredientNameValidationError, Measure, MeasureValidationError};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn add_recipe_ingredient(
    recipe_id: i32,
    ingredient_name: impl TryInto<IngredientName, Error = IngredientNameValidationError> + Debug,
    quantity: f64,
    measure: impl TryInto<Measure, Error = MeasureValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddRecipeIngredient,
) -> Result<RecipeIngredient, AddRecipeIngredientError> {
    let ingredient_name: IngredientName = ingredient_name.try_into()?;
    let measure: Measure = measure.try_into()?;
    let params = AddRecipeIngredientParams::new(
        recipe_id,
        ingredient_name.as_str(),
        quantity,
        measure.as_str(),
        user_id.into(),
    );

    let result = db.add_recipe_ingredient(params).await?;
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct AddRecipeIngredientParams<'a> {
    recipe_id: i32,
    ingredient_name: &'a str,
    quantity: f64,
    measure: &'a str,
    user_id: i32,
}

impl<'a> AddRecipeIngredientParams<'a> {
    fn new(
        recipe_id: i32,
        ingredient_name: &'a str,
        quantity: f64,
        measure: &'a str,
        user_id: UserId,
    ) -> Self {
        Self {
            recipe_id,
            ingredient_name,
            quantity,
            measure,
            user_id: user_id.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AddRecipeIngredientError {
    #[error(transparent)]
    ValidationError(#[from] IngredientNameValidationError),

    #[error(transparent)]
    MeasureValidationError(#[from] MeasureValidationError),

    #[error("Ingredient already exists for this recipe")]
    AlreadyExists,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait AddRecipeIngredient {
    async fn add_recipe_ingredient(
        &self,
        params: AddRecipeIngredientParams<'_>,
    ) -> Result<RecipeIngredient, AddRecipeIngredientError>;
}

impl AddRecipeIngredient for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn add_recipe_ingredient(
        &self,
        params: AddRecipeIngredientParams<'_>,
    ) -> Result<RecipeIngredient, AddRecipeIngredientError> {
        let ingredient_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO ingredients (name)
            VALUES (?)
            ON CONFLICT(name) DO UPDATE SET name = excluded.name
            RETURNING id
            "#,
        )
        .bind(params.ingredient_name)
        .fetch_one(self)
        .instrument(tracing::info_span!("Upsert Ingredient"))
        .await
        .map_err(|e| AddRecipeIngredientError::UnknownDbError(e))?;

        sqlx::query(
            r#"
            INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity, measure)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(params.recipe_id)
        .bind(ingredient_id)
        .bind(params.quantity)
        .bind(params.measure)
        .execute(self)
        .instrument(tracing::info_span!("Insert Recipe Ingredient"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return AddRecipeIngredientError::AlreadyExists;
                }
                AddRecipeIngredientError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => AddRecipeIngredientError::UnknownDbError(e),
        })?;

        Ok(RecipeIngredient {
            recipe_id: params.recipe_id,
            ingredient_id: IngredientId::from(ingredient_id),
            ingredient_name: IngredientName::try_from(params.ingredient_name.to_string())
                .unwrap_or_else(|_| IngredientName::try_from("unknown".to_string()).unwrap()),
            quantity: params.quantity,
            measure: Measure::try_from(params.measure).unwrap_or(Measure::Each),
            raw_ingredient: None,
        })
    }
}
