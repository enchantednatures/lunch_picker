use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::PantryIngredient;
use crate::features::ingredients::{IngredientName, IngredientNameValidationError, Measure, MeasureValidationError};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn update_pantry_ingredient(
    ingredient_name: impl TryInto<IngredientName, Error = IngredientNameValidationError> + Debug,
    quantity: f64,
    measure: impl TryInto<Measure, Error = MeasureValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl UpdatePantryIngredient,
) -> Result<PantryIngredient, UpdatePantryIngredientError> {
    let ingredient_name: IngredientName = ingredient_name.try_into()?;
    let measure: Measure = measure.try_into()?;
    let params = UpdatePantryIngredientParams::new(
        user_id.into(),
        ingredient_name.as_str(),
        quantity,
        measure.as_str(),
    );

    let result = db.update_pantry_ingredient(params).await?;
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct UpdatePantryIngredientParams<'a> {
    user_id: i32,
    ingredient_name: &'a str,
    quantity: f64,
    measure: &'a str,
}

impl<'a> UpdatePantryIngredientParams<'a> {
    fn new(user_id: UserId, ingredient_name: &'a str, quantity: f64, measure: &'a str) -> Self {
        Self {
            user_id: user_id.into(),
            ingredient_name,
            quantity,
            measure,
        }
    }
}

#[derive(Error, Debug)]
pub enum UpdatePantryIngredientError {
    #[error(transparent)]
    ValidationError(#[from] IngredientNameValidationError),

    #[error(transparent)]
    MeasureValidationError(#[from] MeasureValidationError),

    #[error("Ingredient not found in pantry")]
    NotFound,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait UpdatePantryIngredient {
    async fn update_pantry_ingredient(
        &self,
        params: UpdatePantryIngredientParams<'_>,
    ) -> Result<PantryIngredient, UpdatePantryIngredientError>;
}

impl UpdatePantryIngredient for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn update_pantry_ingredient(
        &self,
        params: UpdatePantryIngredientParams<'_>,
    ) -> Result<PantryIngredient, UpdatePantryIngredientError> {
        let result = sqlx::query(
            r#"
            UPDATE pantry_ingredients
            SET quantity = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE user_id = ?
              AND ingredient_id = (SELECT id FROM ingredients WHERE name = ?)
              AND measure = ?
            "#,
        )
        .bind(params.quantity)
        .bind(params.user_id)
        .bind(params.ingredient_name)
        .bind(params.measure)
        .execute(self)
        .instrument(tracing::info_span!("Update Pantry Ingredient"))
        .await
        .map_err(|e| UpdatePantryIngredientError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(UpdatePantryIngredientError::NotFound);
        }

        let row = sqlx::query_as::<_, super::models::PantryIngredientRow>(
            r#"
            SELECT
                pi.ingredient_id,
                i.name as ingredient_name,
                pi.quantity,
                pi.measure
            FROM pantry_ingredients pi
            JOIN ingredients i ON i.id = pi.ingredient_id
            WHERE pi.user_id = ? AND i.name = ? AND pi.measure = ?
            "#,
        )
        .bind(params.user_id)
        .bind(params.ingredient_name)
        .bind(params.measure)
        .fetch_one(self)
        .instrument(tracing::info_span!("Fetch Updated Pantry Ingredient"))
        .await
        .map_err(|e| UpdatePantryIngredientError::UnknownDbError(e))?;

        Ok(row.into())
    }
}
