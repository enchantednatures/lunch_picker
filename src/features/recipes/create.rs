use std::error::Error;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use sqlx::Pool;

use crate::user::UserId;

use super::Recipe;

#[tracing::instrument(skip(db))]
pub async fn create_recipe(
    recipe_name: String,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateRecipe,
) -> Result<Recipe, CreateRecipeError> {
    let recipe = CreateRecipeParams::new(user_id.into(), recipe_name);

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

trait CreateRecipe {
    async fn create_recipe(&self, params: CreateRecipeParams) -> Result<Recipe, CreateRecipeError>;
}
