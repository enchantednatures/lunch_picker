use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::PantryIngredient;
use crate::features::ingredients::{IngredientName, IngredientNameValidationError, Measure, MeasureValidationError};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn add_pantry_ingredient(
    ingredient_name: impl TryInto<IngredientName, Error = IngredientNameValidationError> + Debug,
    quantity: f64,
    measure: impl TryInto<Measure, Error = MeasureValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddPantryIngredient,
) -> Result<PantryIngredient, AddPantryIngredientError> {
    let ingredient_name: IngredientName = ingredient_name.try_into()?;
    let measure: Measure = measure.try_into()?;
    let params = AddPantryIngredientParams::new(
        user_id.into(),
        ingredient_name.as_str(),
        quantity,
        measure.as_str(),
    );

    let result = db.add_pantry_ingredient(params).await?;
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct AddPantryIngredientParams<'a> {
    user_id: i32,
    ingredient_name: &'a str,
    quantity: f64,
    measure: &'a str,
}

impl<'a> AddPantryIngredientParams<'a> {
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
pub enum AddPantryIngredientError {
    #[error(transparent)]
    ValidationError(#[from] IngredientNameValidationError),

    #[error(transparent)]
    MeasureValidationError(#[from] MeasureValidationError),

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait AddPantryIngredient {
    async fn add_pantry_ingredient(
        &self,
        params: AddPantryIngredientParams<'_>,
    ) -> Result<PantryIngredient, AddPantryIngredientError>;
}

impl AddPantryIngredient for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn add_pantry_ingredient(
        &self,
        params: AddPantryIngredientParams<'_>,
    ) -> Result<PantryIngredient, AddPantryIngredientError> {
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
        .map_err(|e| AddPantryIngredientError::UnknownDbError(e))?;

        sqlx::query(
            r#"
            INSERT INTO pantry_ingredients (user_id, ingredient_id, quantity, measure)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(user_id, ingredient_id, measure) DO UPDATE SET
                quantity = quantity + excluded.quantity,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(params.user_id)
        .bind(ingredient_id)
        .bind(params.quantity)
        .bind(params.measure)
        .execute(self)
        .instrument(tracing::info_span!("Upsert Pantry Ingredient"))
        .await
        .map_err(|e| AddPantryIngredientError::UnknownDbError(e))?;

        let row = sqlx::query_as::<_, super::models::PantryIngredientRow>(
            r#"
            SELECT
                pi.ingredient_id,
                i.name as ingredient_name,
                pi.quantity,
                pi.measure
            FROM pantry_ingredients pi
            JOIN ingredients i ON i.id = pi.ingredient_id
            WHERE pi.user_id = ? AND pi.ingredient_id = ? AND pi.measure = ?
            "#,
        )
        .bind(params.user_id)
        .bind(ingredient_id)
        .bind(params.measure)
        .fetch_one(self)
        .instrument(tracing::info_span!("Fetch Pantry Ingredient"))
        .await
        .map_err(|e| AddPantryIngredientError::UnknownDbError(e))?;

        Ok(row.into())
    }
}
