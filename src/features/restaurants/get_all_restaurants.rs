use std::fmt::Debug;

use sqlx::Pool;

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::user::UserId;

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

#[cfg(feature = "postgres")]
impl GetAllRestaurants for Pool<Postgres> {
    #[tracing::instrument(name = "Getting all Restaurants", skip(self))]
    async fn get_all_restaurants(&self, params: UserId) -> Result<Vec<Restaurant>, sqlx::Error> {
        let restaurant: Vec<RestaurantRow> =
            sqlx::query_as(r#"select id, user_id, name from restaurants where user_id = $1"#)
                .bind(params.as_i32())
                .fetch_all(self)
                .instrument(tracing::info_span!("Querying all restaurants"))
                .await?;
        Ok(restaurant.into_iter().map(|x| x.into()).collect())
    }
}

#[cfg(feature = "sqlite")]
impl GetAllRestaurants for Pool<Sqlite> {
    #[tracing::instrument(name = "Getting all Restaurants", skip(self))]
    async fn get_all_restaurants(&self, params: UserId) -> Result<Vec<Restaurant>, sqlx::Error> {
        let restaurant: Vec<RestaurantRow> =
            sqlx::query_as(r#"select id, user_id, name from restaurants where user_id = ?"#)
                .bind(params.as_i32())
                .fetch_all(self)
                .instrument(tracing::info_span!("Querying all restaurants"))
                .await?;
        Ok(restaurant.into_iter().map(|x| x.into()).collect())
    }
}
