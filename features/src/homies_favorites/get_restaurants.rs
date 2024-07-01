use std::fmt::Debug;

use thiserror::Error;
use tracing::Instrument;

use crate::{HomieId, Restaurant, RestaurantRow};
use models::UserId;

#[tracing::instrument(name = "Getting all Restaurants", skip(db))]
pub async fn get_homies_favorite_restaurants(
    user_id: impl Into<UserId> + Debug,
    homie_id: impl Into<HomieId> + Debug,
    db: &impl GetHomiesFavoriteRestaurants,
) -> Result<Vec<Restaurant>, GetHomiesFavoriteRestaurantsError> {
    let params = GetHomiesFavoriteRestaurantsParams::new(user_id.into(), homie_id.into());
    let retrieved_restaurants = db.get_homies_favorite_restaurants(&params).await?;

    Ok(retrieved_restaurants)
}

#[derive(Debug)]
struct GetHomiesFavoriteRestaurantsParams {
    user_id: UserId,
    homie_id: HomieId,
}

impl GetHomiesFavoriteRestaurantsParams {
    fn new(user_id: UserId, homie_id: HomieId) -> Self {
        Self { user_id, homie_id }
    }
}

#[derive(Error, Debug)]
pub enum GetHomiesFavoriteRestaurantsError {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

// todo: make this not return a sqlx error
pub trait GetHomiesFavoriteRestaurants {
    async fn get_homies_favorite_restaurants(
        &self,
        params: &GetHomiesFavoriteRestaurantsParams,
    ) -> Result<Vec<Restaurant>, sqlx::Error>;
}

