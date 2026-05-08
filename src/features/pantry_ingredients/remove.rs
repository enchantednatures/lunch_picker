use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::features::ingredients::{IngredientName, IngredientNameValidationError, Measure, MeasureValidationError};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn remove_pantry_ingredient(
    ingredient_name: impl TryInto<IngredientName, Error = IngredientNameValidationError> + Debug,
    measure: Option<impl TryInto<Measure, Error = MeasureValidationError> + Debug>,
    user_id: impl Into<UserId> + Debug,
    db: &impl RemovePantryIngredient,
) -> Result<(), RemovePantryIngredientError> {
    let ingredient_name: IngredientName = ingredient_name.try_into()?;
    let measure_str = measure
        .map(|m| {
            let measure: Measure = m.try_into()?;
            Ok::<_, MeasureValidationError>(measure.as_str().to_string())
        })
        .transpose()?;

    let params = RemovePantryIngredientParams::new(
        user_id.into(),
        ingredient_name.as_str(),
        measure_str.as_deref(),
    );

    db.remove_pantry_ingredient(params).await?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct RemovePantryIngredientParams<'a> {
    user_id: i32,
    ingredient_name: &'a str,
    measure: Option<&'a str>,
}

impl<'a> RemovePantryIngredientParams<'a> {
    fn new(user_id: UserId, ingredient_name: &'a str, measure: Option<&'a str>) -> Self {
        Self {
            user_id: user_id.into(),
            ingredient_name,
            measure,
        }
    }
}

#[derive(Error, Debug)]
pub enum RemovePantryIngredientError {
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

pub(crate) trait RemovePantryIngredient {
    async fn remove_pantry_ingredient(
        &self,
        params: RemovePantryIngredientParams<'_>,
    ) -> Result<(), RemovePantryIngredientError>;
}

impl RemovePantryIngredient for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn remove_pantry_ingredient(
        &self,
        params: RemovePantryIngredientParams<'_>,
    ) -> Result<(), RemovePantryIngredientError> {
        let query = if let Some(measure) = params.measure {
            sqlx::query(
                r#"
                DELETE FROM pantry_ingredients
                WHERE user_id = ?
                  AND ingredient_id = (SELECT id FROM ingredients WHERE name = ?)
                  AND measure = ?
                "#,
            )
            .bind(params.user_id)
            .bind(params.ingredient_name)
            .bind(measure)
        } else {
            sqlx::query(
                r#"
                DELETE FROM pantry_ingredients
                WHERE user_id = ?
                  AND ingredient_id = (SELECT id FROM ingredients WHERE name = ?)
                "#,
            )
            .bind(params.user_id)
            .bind(params.ingredient_name)
        };

        let result = query
            .execute(self)
            .instrument(tracing::info_span!("Remove Pantry Ingredient"))
            .await
            .map_err(|e| RemovePantryIngredientError::UnknownDbError(e))?;

        if result.rows_affected() == 0 {
            return Err(RemovePantryIngredientError::NotFound);
        }

        Ok(())
    }
}
