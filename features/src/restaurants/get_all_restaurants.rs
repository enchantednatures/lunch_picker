use std::fmt::Debug;

use models::UserId;
use thiserror::Error;
use tracing::Instrument;


use super::Restaurant;
use super::RestaurantRow;

#[tracing::instrument(name = "Getting all Restaurants", skip(db))]
pub async fn get_all_restaurants(
    user_id: impl Into<UserId> + Debug,
    db: &impl GetAllRestaurants,
) -> Result<Vec<Restaurant>, GetAllRestaurantsError> {
    let retrieved_restaurants = db.get_all_restaurants(user_id.into()).await?;

    Ok(retrieved_restaurants)
}
#[derive(Error, Debug)]
pub enum GetAllRestaurantsError {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

pub trait GetAllRestaurants {
    async fn get_all_restaurants(&self, params: UserId) -> Result<Vec<Restaurant>, sqlx::Error>;
}
