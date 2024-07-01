use std::fmt::Debug;

use models::UserId;
use thiserror::Error;
use tracing::Instrument;

use super::models::Restaurant;
use super::RestaurantRow;

#[tracing::instrument(skip(db))]
pub async fn create_restaurant(
    restaurant_name: String,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateRestaurant,
) -> Result<Restaurant, CreateRestaurantError> {
    // todo: validate name 
    let restaurant = CreateRestaurantParams::new(user_id.into(), &restaurant_name);

    let created_restaurant = db.create_restaurant(restaurant).await?;

    Ok(created_restaurant)
}

#[derive(Debug)]
pub struct CreateRestaurantParams<'a> {
    user_id: i32,
    name: &'a str,
}

impl<'a> CreateRestaurantParams<'a> {
    fn new(user_id: UserId, name: &'a str) -> Self {
        Self {
            user_id: user_id.into(),
            name,
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateRestaurantError {
    #[error("Invalid Name")]
    InvalidName { name: String },

    #[error("Invalid User: {:?}", constraint)]
    ForeignKeyViolation { constraint: String },

    #[error("Restaurant already exists: {:?}", name)]
    RestaurantAlreadyExists { name: String },

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub trait CreateRestaurant {
    async fn create_restaurant<'a>(
        &self,
        params: CreateRestaurantParams<'a>,
    ) -> Result<Restaurant, CreateRestaurantError>;
}
