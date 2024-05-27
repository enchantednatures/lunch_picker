use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;
use sqlx::Sqlite;
use tracing::info;

use crate::validator::Validator;

#[tracing::instrument]
pub async fn create_recipe(
    recipe_name: String,
    db: &Pool<Sqlite>,
) -> Result<Recipe, CreateRecipeError> {
    info!("Creating recipe");

    let recipe: CreateRecipeParams = recipe_name.into();
    recipe.validate()?;
    let created_recipe = db.create_recipe(recipe).await?;

    Ok(created_recipe)
}
impl Error for CreateRecipeError {}

#[derive(Debug)]
struct CreateRecipeParams {
    name: String,
}
impl CreateRecipeParams {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl From<String> for CreateRecipeParams {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

enum CreateRecipeParamsError {
    InvalidName,
}

enum CreateRecipeDbError {
    RecipeAlreadyExists,
    Unknown,
}

#[derive(Debug)]
pub enum CreateRecipeError {
    InvalidName,
    RecipeAlreadyExists,
    Unknown,
}

impl Display for CreateRecipeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CreateRecipeError::InvalidName => write!(f, "Invalid name"),
            CreateRecipeError::RecipeAlreadyExists => write!(f, "Recipe already exists"),
            CreateRecipeError::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl From<CreateRecipeDbError> for CreateRecipeError {
    fn from(value: CreateRecipeDbError) -> Self {
        match value {
            CreateRecipeDbError::RecipeAlreadyExists => CreateRecipeError::RecipeAlreadyExists,
            CreateRecipeDbError::Unknown => CreateRecipeError::Unknown,
        }
    }
}
impl From<CreateRecipeParamsError> for CreateRecipeError {
    fn from(value: CreateRecipeParamsError) -> Self {
        match value {
            CreateRecipeParamsError::InvalidName => CreateRecipeError::InvalidName,
        }
    }
}

impl Validator for CreateRecipeParams {
    type E = CreateRecipeParamsError;

    fn validate(&self) -> Result<(), Self::E> {
        match self.name.is_empty() {
            true => return Err(CreateRecipeParamsError::InvalidName),
            false => return Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
}

trait CreateRecipe {
    async fn create_recipe(
        &self,
        params: CreateRecipeParams,
    ) -> Result<Recipe, CreateRecipeDbError>;
}

impl CreateRecipe for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn create_recipe(
        &self,
        params: CreateRecipeParams,
    ) -> Result<Recipe, CreateRecipeDbError> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"INSERT INTO recipes (name) VALUES (?) RETURNING id, name"#,
            params.name
        )
        .fetch_one(self)
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);
            match e {
                sqlx::Error::Database(db_error) => {
                    if db_error.is_unique_violation() {
                        return CreateRecipeDbError::RecipeAlreadyExists;
                    }
                    CreateRecipeDbError::Unknown
                }
                _ => CreateRecipeDbError::Unknown,
            }
        })?;
        Ok(recipe)
    }
}
