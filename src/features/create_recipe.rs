use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;
use sqlx::Postgres;

use crate::user::UserId;
use crate::validator::Validator;

#[tracing::instrument(skip(db))]
pub async fn create_recipe(
    recipe_name: String,
    user_id: UserId,
    db: &Pool<Postgres>,
) -> Result<Recipe, CreateRecipeError> {
    let recipe = CreateRecipeParams::new(user_id, recipe_name);

    recipe.validate()?;
    let created_recipe = db.create_recipe(recipe).await?;

    Ok(created_recipe)
}

impl Error for CreateRecipeError {}

#[derive(Debug)]
struct CreateRecipeParams {
    user_id: i32,
    name: String,
}

impl CreateRecipeParams {
    fn new(user_id: UserId, name: String) -> Self {
        Self {
            user_id: user_id.into(),
            name,
        }
    }
}

enum CreateRecipeParamsError {
    InvalidName,
}

enum CreateRecipeDbError {
    Unknown,
    ForeignKeyViolation { constraint: String },
    RecipeAlreadyExists,
}

#[derive(Debug)]
pub enum CreateRecipeError {
    Unknown,
    UnknownDbError(String),
    InvalidName,
    ForeignKeyViolation { constraint: String },
    RecipeAlreadyExists,
}

impl Display for CreateRecipeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CreateRecipeError::InvalidName => write!(f, "Invalid name"),
            CreateRecipeError::RecipeAlreadyExists => write!(f, "Recipe already exists"),
            CreateRecipeError::Unknown => write!(f, "Unknown error"),
            CreateRecipeError::UnknownDbError(e) => write!(f, "Unknown db error: {}", e),
            CreateRecipeError::ForeignKeyViolation { constraint } => {
                write!(f, "Foreign key violation: {}", constraint)
            }
        }
    }
}

impl From<CreateRecipeDbError> for CreateRecipeError {
    fn from(value: CreateRecipeDbError) -> Self {
        match value {
            CreateRecipeDbError::RecipeAlreadyExists => CreateRecipeError::RecipeAlreadyExists,
            CreateRecipeDbError::Unknown => CreateRecipeError::Unknown,
            CreateRecipeDbError::ForeignKeyViolation { constraint } => CreateRecipeError::ForeignKeyViolation { constraint }
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

    #[tracing::instrument]
    fn validate(&self) -> Result<(), Self::E> {
        match self.name.is_empty() {
            true => Err(CreateRecipeParamsError::InvalidName),
            false => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
}

trait CreateRecipe {
    async fn create_recipe(&self, params: CreateRecipeParams) -> Result<Recipe, CreateRecipeDbError>;
}

impl CreateRecipe for Pool<Postgres> {
    #[tracing::instrument(skip(self))]
    async fn create_recipe(&self, params: CreateRecipeParams) -> Result<Recipe, CreateRecipeDbError> {
        let recipe = sqlx::query_as!(
            Recipe,
            r#"INSERT INTO recipes (user_id, name) VALUES ($1, $2) RETURNING id, name"#,
            params.user_id,
            params.name
        )
        .fetch_one(self)
        .await
        .map_err(|e| {
            match e {
                sqlx::Error::Database(db_error) => {
                    if db_error.is_unique_violation() {
                        return CreateRecipeDbError::RecipeAlreadyExists;
                    } else if db_error.is_foreign_key_violation() {
                        return CreateRecipeDbError::ForeignKeyViolation {
                            constraint: db_error
                                .constraint()
                                .expect("Constraint should be named if it is a ForeignKeyViolation")
                                .to_string(),
                        };
                    }
                    CreateRecipeDbError::Unknown
                }
                _ => CreateRecipeDbError::Unknown,
            }
        })?;
        Ok(recipe)
    }
}

