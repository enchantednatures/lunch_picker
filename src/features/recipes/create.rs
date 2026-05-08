use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::{Recipe, RecipeName, RecipeNameValidationError, RecipeRow};
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn create_recipe(
    recipe_name: impl TryInto<RecipeName, Error = RecipeNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    description: Option<String>,
    prep_time: Option<i32>,
    cook_time: Option<i32>,
    servings: Option<i32>,
    db: &impl CreateRecipe,
) -> Result<Recipe, CreateRecipeError> {
    let recipe_name: RecipeName = recipe_name.try_into()?;
    let params = CreateRecipeParams::new(
        user_id.into(),
        recipe_name.as_str(),
        description,
        prep_time,
        cook_time,
        servings,
    );

    let created_recipe = db.create_recipe(params).await?;
    Ok(created_recipe)
}

#[derive(Debug)]
pub(crate) struct CreateRecipeParams<'a> {
    user_id: i32,
    name: &'a str,
    description: Option<String>,
    prep_time: Option<i32>,
    cook_time: Option<i32>,
    servings: Option<i32>,
}

impl<'a> CreateRecipeParams<'a> {
    fn new(
        user_id: UserId,
        name: &'a str,
        description: Option<String>,
        prep_time: Option<i32>,
        cook_time: Option<i32>,
        servings: Option<i32>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            name,
            description,
            prep_time,
            cook_time,
            servings,
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateRecipeError {
    #[error(transparent)]
    ValidationError(#[from] RecipeNameValidationError),

    #[error("Invalid User: {constraint}")]
    ForeignKeyViolation { constraint: String },

    #[error("Recipe already exists: {name}")]
    RecipeAlreadyExists { name: String },

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait CreateRecipe {
    async fn create_recipe(
        &self,
        params: CreateRecipeParams<'_>,
    ) -> Result<Recipe, CreateRecipeError>;
}

impl CreateRecipe for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_recipe(
        &self,
        params: CreateRecipeParams<'_>,
    ) -> Result<Recipe, CreateRecipeError> {
        let recipe: RecipeRow = sqlx::query_as(
            r#"
            INSERT INTO recipes (user_id, name, description, prep_time, cook_time, servings)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, user_id, name, description, prep_time, cook_time, servings, source_url, imported_at
            "#,
        )
        .bind(params.user_id)
        .bind(params.name)
        .bind(params.description)
        .bind(params.prep_time)
        .bind(params.cook_time)
        .bind(params.servings)
        .fetch_one(self)
        .instrument(tracing::info_span!("Insert Recipe"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateRecipeError::RecipeAlreadyExists {
                        name: params.name.to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return CreateRecipeError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                CreateRecipeError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateRecipeError::UnknownDbError(e),
        })?;

        Ok(recipe.into())
    }
}
