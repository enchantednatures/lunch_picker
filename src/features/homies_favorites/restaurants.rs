use std::fmt::Debug;

use sqlx::{Pool, Postgres};
use thiserror::Error;
use tracing::Instrument;

use crate::features::HomieNameValidationError;
use crate::features::HomiesName;
use crate::features::RestaurantName;
use crate::features::RestaurantNameValidationError;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn add_homies_favorite_restaurant(
    homie_name: impl TryInto<HomiesName, Error = HomieNameValidationError> + Debug,
    restaurant_name: impl TryInto<RestaurantName, Error = RestaurantNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddFavoriteRestaurantToHomie,
) -> Result<(), AddHomiesFavoriteRestaurantError> {
    let add_favorite_to_homie_params = AddFavoriteRestaurantToHomieParams::new(
        user_id.into(),
        homie_name.try_into()?,
        restaurant_name.try_into()?,
    );

    db.add_homies_favorite_restaurant(&add_favorite_to_homie_params)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return AddHomiesFavoriteRestaurantError::HomieAlreadyHasFavorite {
                        name: add_favorite_to_homie_params.name.as_str().to_string(),
                        restaurant_name: add_favorite_to_homie_params
                            .restaurant_name
                            .as_str()
                            .to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return AddHomiesFavoriteRestaurantError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                AddHomiesFavoriteRestaurantError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            sqlx::Error::RowNotFound => AddHomiesFavoriteRestaurantError::NoFavoriteAdded,
            _ => AddHomiesFavoriteRestaurantError::UnknownDbError(e),
        })?;

    Ok(())
}

#[derive(Debug)]
struct AddFavoriteRestaurantToHomieParams {
    user_id: UserId,
    name: HomiesName,
    restaurant_name: RestaurantName,
}

impl AddFavoriteRestaurantToHomieParams {
    fn new(user_id: UserId, name: HomiesName, restaurant_name: RestaurantName) -> Self {
        Self {
            user_id,
            name,
            restaurant_name,
        }
    }
}

#[derive(Error, Debug)]
pub enum AddHomiesFavoriteRestaurantError {
    #[error(transparent)]
    HomieNameValidationError(#[from] HomieNameValidationError),

    #[error(transparent)]
    RestaurantNameValidationError(#[from] RestaurantNameValidationError),

    #[error("No favorite added")]
    NoFavoriteAdded,

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

trait AddFavoriteRestaurantToHomie {
    async fn add_homies_favorite_restaurant<'a>(
        &self,
        params: &AddFavoriteRestaurantToHomieParams,
    ) -> Result<(), sqlx::Error>;
}

impl AddFavoriteRestaurantToHomie for Pool<Postgres> {
    #[tracing::instrument(skip(self))]
    async fn add_homies_favorite_restaurant<'a>(
        &self,
        params: &AddFavoriteRestaurantToHomieParams,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            r#"
                insert into homies_favorite_restaurants (homie_id, user_id, restaurant_id)
                select 
                    h.id, 
                    $1,
                    r.id
                from homies h
                join restaurants r on r.name = $3 and r.user_id = $1
                where h.name = $2 and h.user_id = $1
                limit 1
                returning *;
                ;
            "#,
            params.user_id.as_i32(),
            params.name.as_str(),
            params.restaurant_name.as_str()
        )
        .fetch_one(self)
        .instrument(tracing::info_span!(
            "Adding favorite restaurant to homie db query"
        ))
        .await?;
        Ok(())
    }
}
