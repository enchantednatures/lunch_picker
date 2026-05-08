use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{PantryIngredient, PantryIngredientRow};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn list_pantry_ingredients(
    user_id: impl Into<UserId> + std::fmt::Debug,
    db: &impl ListPantryIngredients,
) -> Result<Vec<PantryIngredient>, ListPantryIngredientsError> {
    let rows = db.list_pantry_ingredients(user_id.into()).await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

#[derive(Error, Debug)]
pub enum ListPantryIngredientsError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ListPantryIngredients {
    async fn list_pantry_ingredients(
        &self,
        user_id: UserId,
    ) -> Result<Vec<PantryIngredientRow>, ListPantryIngredientsError>;
}

impl ListPantryIngredients for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn list_pantry_ingredients(
        &self,
        user_id: UserId,
    ) -> Result<Vec<PantryIngredientRow>, ListPantryIngredientsError> {
        let rows = sqlx::query_as::<_, PantryIngredientRow>(
            r#"
            SELECT
                pi.ingredient_id,
                i.name as ingredient_name,
                pi.quantity,
                pi.measure
            FROM pantry_ingredients pi
            JOIN ingredients i ON i.id = pi.ingredient_id
            WHERE pi.user_id = ?
            ORDER BY i.name
            "#,
        )
        .bind(user_id.as_i32())
        .fetch_all(self)
        .instrument(tracing::info_span!("List Pantry Ingredients"))
        .await
        .map_err(|e| ListPantryIngredientsError::UnknownDbError(e))?;

        Ok(rows)
    }
}
