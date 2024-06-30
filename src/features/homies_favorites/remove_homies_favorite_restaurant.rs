use std::fmt::Debug;

use sqlx::Pool;

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use crate::features::HomieNameValidationError;
use crate::features::HomiesName;
use crate::features::RestaurantName;
use crate::features::RestaurantNameValidationError;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn remove_homies_favorite_restaurant(
    homie_name: impl TryInto<HomiesName, Error = HomieNameValidationError> + Debug,
    restaurant_name: impl TryInto<RestaurantName, Error = RestaurantNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl RemoveFavoriteRestaurantFromHomie,
) -> Result<(), RemoveHomiesFavoriteRestaurantError> {
    let remove_favorite_from_homie_params = RemoveFavoriteRestaurantFromHomieParams::new(
        user_id.into(),
        homie_name.try_into()?,
        restaurant_name.try_into()?,
    );

    db.remove_homies_favorite_restaurant(&remove_favorite_from_homie_params)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return RemoveHomiesFavoriteRestaurantError::HomieAlreadyHasFavorite {
                        name: remove_favorite_from_homie_params.name.as_str().to_string(),
                        restaurant_name: remove_favorite_from_homie_params
                            .restaurant_name
                            .as_str()
                            .to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return RemoveHomiesFavoriteRestaurantError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                RemoveHomiesFavoriteRestaurantError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            sqlx::Error::RowNotFound => RemoveHomiesFavoriteRestaurantError::NoFavoriteRemoved,
            _ => RemoveHomiesFavoriteRestaurantError::UnknownDbError(e),
        })?;

    Ok(())
}

#[derive(Debug)]
struct RemoveFavoriteRestaurantFromHomieParams {
    user_id: UserId,
    name: HomiesName,
    restaurant_name: RestaurantName,
}

impl RemoveFavoriteRestaurantFromHomieParams {
    fn new(user_id: UserId, name: HomiesName, restaurant_name: RestaurantName) -> Self {
        Self {
            user_id,
            name,
            restaurant_name,
        }
    }
}

#[derive(Error, Debug)]
pub enum RemoveHomiesFavoriteRestaurantError {
    #[error(transparent)]
    HomieNameValidationError(#[from] HomieNameValidationError),

    #[error(transparent)]
    RestaurantNameValidationError(#[from] RestaurantNameValidationError),

    #[error("No favorite removed")]
    NoFavoriteRemoved,

    #[error("Invalid User")]
    ForeignKeyViolation { constraint: String },

    #[error("{:?} already has {:?} favorited", name, restaurant_name)]
    HomieAlreadyHasFavorite {
        name: String,
        restaurant_name: String,
    },

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub trait RemoveFavoriteRestaurantFromHomie {
    async fn remove_homies_favorite_restaurant<'a>(
        &self,
        params: &RemoveFavoriteRestaurantFromHomieParams,
    ) -> Result<(), sqlx::Error>;
}

#[cfg(feature = "postgres")]
impl RemoveFavoriteRestaurantFromHomie for Pool<Postgres> {
    #[tracing::instrument(skip(self))]
    async fn remove_homies_favorite_restaurant<'a>(
        &self,
        params: &RemoveFavoriteRestaurantFromHomieParams,
    ) -> Result<(), sqlx::Error> {
        dbg!(params);
        _ = sqlx::query!(
            r#"
delete
from homies_favorite_restaurants t

where exists (select distinct 1
              from homies_favorite_restaurants f
                       inner join homies h on h.name = $2 and h.id = f.homie_id
                       inner join restaurants r on r.name = $3 and r.id = f.restaurant_id
              where f.user_id = $1 
                and t.user_id = f.user_id
                and t.homie_id = f.homie_id
                and t.restaurant_id = f.restaurant_id);
"#,
            params.user_id.as_i32(),
            params.name.as_str(),
            params.restaurant_name.as_str()
        )
        .execute(self)
        .instrument(tracing::info_span!(
            "Removing favorite restaurant to homie db query"
        ))
        .await?;
        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl RemoveFavoriteRestaurantFromHomie for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn remove_homies_favorite_restaurant<'a>(
        &self,
        params: &RemoveFavoriteRestaurantFromHomieParams,
    ) -> Result<(), sqlx::Error> {
        let user_id = params.user_id.as_i32();
        let restaurant_name = params.restaurant_name.as_str();
        todo!();
        let homie_name = params.name.as_str();
        _ = sqlx::query!(
            r#"
select *
from homies_favorite_restaurants
where exists (select homies.id,
                     1,
                     restaurants.id
              from homies
                       join restaurants on restaurants.name = ? and restaurants.user_id = 1
              where homies.name = ?
                and homies.user_id = 1
                and homies_favorite_restaurants.homie_id = homies.id
                and homies_favorite_restaurants.restaurant_id = restaurants.id
                and homies_favorite_restaurants.user_id = ?);
                ;
            "#,
            user_id,
            restaurant_name,
            user_id,
            homie_name,
            user_id
        )
        .fetch_one(self)
        .instrument(tracing::info_span!(
            "Removeing favorite restaurant to homie db query"
        ))
        .await?;
        Ok(())
    }
}
