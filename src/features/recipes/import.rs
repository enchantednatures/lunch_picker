use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{Recipe, RecipeName, RecipeRow};
use crate::user::UserId;

mod ingredient_parser;
mod schema_org;

pub use ingredient_parser::*;
pub use schema_org::*;

#[tracing::instrument(skip(db))]
pub async fn import_recipe_from_url(
    url: String,
    user_id: impl Into<UserId> + Debug,
    db: &impl ImportRecipeFromUrl,
) -> Result<Recipe, ImportRecipeError> {
    let user_id = user_id.into();

    let existing = db.find_recipe_by_url(&url, user_id).await?;
    if existing.is_some() {
        return Err(ImportRecipeError::AlreadyImported);
    }

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ImportRecipeError::HttpError(e.to_string()))?;

    let html = response
        .text()
        .await
        .map_err(|e| ImportRecipeError::HttpError(e.to_string()))?;

    let parsed = schema_org::extract_recipe_from_html(&html, &url)
        .ok_or(ImportRecipeError::NoRecipeData)?;

    let params = ImportRecipeParams {
        user_id: user_id.into(),
        name: parsed.name,
        description: parsed.description,
        prep_time: parsed.prep_time_minutes,
        cook_time: parsed.cook_time_minutes,
        servings: parsed.servings,
        source_url: url,
        ingredients: parsed.ingredients,
        instructions: parsed.instructions,
    };

    let recipe = db.import_recipe(params).await?;
    Ok(recipe)
}

#[derive(Debug)]
pub(crate) struct ImportRecipeParams {
    user_id: i32,
    name: String,
    description: Option<String>,
    prep_time: Option<i32>,
    cook_time: Option<i32>,
    servings: Option<i32>,
    source_url: String,
    ingredients: Vec<String>,
    instructions: Vec<String>,
}

#[derive(Error, Debug)]
pub enum ImportRecipeError {
    #[error("Recipe already imported from this URL")]
    AlreadyImported,

    #[error("Failed to fetch URL: {0}")]
    HttpError(String),

    #[error("No recipe data found at this URL")]
    NoRecipeData,

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait ImportRecipeFromUrl {
    async fn find_recipe_by_url(
        &self,
        url: &str,
        user_id: UserId,
    ) -> Result<Option<RecipeRow>, sqlx::Error>;

    async fn import_recipe(
        &self,
        params: ImportRecipeParams,
    ) -> Result<Recipe, ImportRecipeError>;
}

impl ImportRecipeFromUrl for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn find_recipe_by_url(
        &self,
        url: &str,
        user_id: UserId,
    ) -> Result<Option<RecipeRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, user_id, name, description, prep_time, cook_time, servings, source_url, imported_at
            FROM recipes
            WHERE user_id = ? AND source_url = ?
            "#,
        )
        .bind(user_id.as_i32())
        .bind(url)
        .fetch_optional(self)
        .instrument(tracing::info_span!("Find Recipe By URL"))
        .await?;

        Ok(row)
    }

    #[tracing::instrument(skip(self, params))]
    async fn import_recipe(
        &self,
        params: ImportRecipeParams,
    ) -> Result<Recipe, ImportRecipeError> {
        let mut tx = self.begin().await.map_err(|e| ImportRecipeError::UnknownDbError(e))?;

        let recipe_row: RecipeRow = sqlx::query_as(
            r#"
            INSERT INTO recipes (user_id, name, description, prep_time, cook_time, servings, source_url, imported_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            RETURNING id, user_id, name, description, prep_time, cook_time, servings, source_url, imported_at
            "#,
        )
        .bind(params.user_id)
        .bind(&params.name)
        .bind(params.description)
        .bind(params.prep_time)
        .bind(params.cook_time)
        .bind(params.servings)
        .bind(&params.source_url)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return ImportRecipeError::AlreadyImported;
                }
                ImportRecipeError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => ImportRecipeError::UnknownDbError(e),
        })?;

        let recipe_id = recipe_row.id;

        for (i, instruction) in params.instructions.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO recipe_steps (recipe_id, user_id, step_number, instruction)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(recipe_id)
            .bind(params.user_id)
            .bind((i + 1) as i32)
            .bind(instruction)
            .execute(&mut *tx)
            .await
            .map_err(|e| ImportRecipeError::UnknownDbError(e))?;
        }

        for ingredient_str in params.ingredients {
            match ingredient_parser::parse_ingredient(&ingredient_str) {
                Ok(parsed) => {
                    let ingredient_id: i32 = sqlx::query_scalar(
                        r#"
                        INSERT INTO ingredients (name)
                        VALUES (?)
                        ON CONFLICT(name) DO UPDATE SET name = excluded.name
                        RETURNING id
                        "#,
                    )
                    .bind(&parsed.name)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| ImportRecipeError::UnknownDbError(e))?;

                    sqlx::query(
                        r#"
                        INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity, measure)
                        VALUES (?, ?, ?, ?)
                        "#,
                    )
                    .bind(recipe_id)
                    .bind(ingredient_id)
                    .bind(parsed.quantity)
                    .bind(parsed.measure.as_str())
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ImportRecipeError::UnknownDbError(e))?;
                }
                Err(_) => {
                    let ingredient_id: i32 = sqlx::query_scalar(
                        r#"
                        INSERT INTO ingredients (name)
                        VALUES (?)
                        ON CONFLICT(name) DO UPDATE SET name = excluded.name
                        RETURNING id
                        "#,
                    )
                    .bind(&ingredient_str)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| ImportRecipeError::UnknownDbError(e))?;

                    sqlx::query(
                        r#"
                        INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity, measure, raw_ingredient)
                        VALUES (?, ?, ?, ?, ?)
                        "#,
                    )
                    .bind(recipe_id)
                    .bind(ingredient_id)
                    .bind(1.0)
                    .bind("each")
                    .bind(&ingredient_str)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| ImportRecipeError::UnknownDbError(e))?;
                }
            }
        }

        tx.commit()
            .await
            .map_err(|e| ImportRecipeError::UnknownDbError(e))?;

        Ok(recipe_row.into())
    }
}
