use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

#[derive(Debug)]
pub struct ShoppingListItem {
    pub ingredient_name: String,
    pub measure: String,
    pub quantity_to_buy: f64,
}

#[tracing::instrument(skip(db))]
pub async fn generate_shopping_list(
    user_id: impl Into<UserId> + std::fmt::Debug,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    db: &impl GenerateShoppingList,
) -> Result<Vec<ShoppingListItem>, GenerateShoppingListError> {
    let items = db
        .generate_shopping_list(user_id.into(), start_date, end_date)
        .await?;
    Ok(items)
}

#[derive(Error, Debug)]
pub enum GenerateShoppingListError {
    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait GenerateShoppingList {
    async fn generate_shopping_list(
        &self,
        user_id: UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<ShoppingListItem>, GenerateShoppingListError>;
}

impl GenerateShoppingList for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn generate_shopping_list(
        &self,
        user_id: UserId,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<ShoppingListItem>, GenerateShoppingListError> {
        let rows = sqlx::query_as::<_, ShoppingListItemRow>(
            r#"
            WITH planned_recipes AS (
                SELECT recipe_id
                FROM meal_plan_entries
                WHERE user_id = ?
                  AND date BETWEEN ? AND ?
            ),
            needed AS (
                SELECT
                    ri.ingredient_id,
                    ri.measure,
                    SUM(ri.quantity) AS total_needed
                FROM recipe_ingredients ri
                JOIN planned_recipes pr ON ri.recipe_id = pr.recipe_id
                GROUP BY ri.ingredient_id, ri.measure
            ),
            pantry AS (
                SELECT ingredient_id, measure, quantity AS total_have
                FROM pantry_ingredients
                WHERE user_id = ?
            )
            SELECT
                i.name as ingredient_name,
                n.measure,
                MAX(0.0, n.total_needed - COALESCE(p.total_have, 0.0)) AS quantity_to_buy
            FROM needed n
            JOIN ingredients i ON n.ingredient_id = i.id
            LEFT JOIN pantry p ON n.ingredient_id = p.ingredient_id AND n.measure = p.measure
            WHERE n.total_needed > COALESCE(p.total_have, 0.0)
            ORDER BY i.name
            "#,
        )
        .bind(user_id.as_i32())
        .bind(start_date)
        .bind(end_date)
        .bind(user_id.as_i32())
        .fetch_all(self)
        .instrument(tracing::info_span!("Generate Shopping List"))
        .await
        .map_err(|e| GenerateShoppingListError::UnknownDbError(e))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct ShoppingListItemRow {
    ingredient_name: String,
    measure: String,
    quantity_to_buy: f64,
}

impl From<ShoppingListItemRow> for ShoppingListItem {
    fn from(row: ShoppingListItemRow) -> Self {
        Self {
            ingredient_name: row.ingredient_name,
            measure: row.measure,
            quantity_to_buy: row.quantity_to_buy,
        }
    }
}
