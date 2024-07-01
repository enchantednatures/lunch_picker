use std::fmt::Debug;

use sqlx::Pool;

use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::features::{HomieId, Restaurant, RestaurantRow};
use crate::user::UserId;

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

// todo: do we need the second user_id clause?
// todo: does this only need to return a restaurant id?
pub trait GetHomiesFavoriteRestaurants {
    async fn get_homies_favorite_restaurants(
        &self,
        params: &GetHomiesFavoriteRestaurantsParams,
    ) -> Result<Vec<Restaurant>, sqlx::Error>;
}

// todo: do we need the second user_id clause?
// todo: does this only need to return a restaurant id?
impl GetHomiesFavoriteRestaurants for Pool<Sqlite> {
    #[tracing::instrument(name = "Getting all Restaurants", skip(self))]
    async fn get_homies_favorite_restaurants(
        &self,
        params: &GetHomiesFavoriteRestaurantsParams,
    ) -> Result<Vec<Restaurant>, sqlx::Error> {
        let restaurant: Vec<RestaurantRow> = sqlx::query_as(
            r#"
select id, r.user_id, name
from restaurants r
         inner join homies_favorite_restaurants
              on r.id = homies_favorite_restaurants.restaurant_id
                  and homies_favorite_restaurants.homie_id = ?
                  and homies_favorite_restaurants.user_id = ? 
where r.user_id = ? 
"#,
        )
        .bind(params.homie_id.as_i32())
        .bind(params.user_id.as_i32())
        .bind(params.user_id.as_i32())
        .fetch_all(self)
        .instrument(tracing::info_span!("Querying all restaurants"))
        .await?;
        Ok(restaurant.into_iter().map(|x| x.into()).collect())
    }
}
