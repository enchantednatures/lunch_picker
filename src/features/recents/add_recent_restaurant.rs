use std::fmt::Debug;

use sqlx::{Pool, Postgres};
use thiserror::Error;
use tracing::Instrument;

use crate::features::HomieId;
use crate::features::HomieNameValidationError;
use crate::features::HomiesName;
use crate::features::RestaurantId;
use crate::features::RestaurantName;
use crate::features::RestaurantNameValidationError;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn add_recent_restaurant_for_homie(
    homie_name: impl TryInto<HomiesName, Error = HomieNameValidationError> + Debug,
    restaurant_name: impl TryInto<RestaurantName, Error = RestaurantNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddRecentRestaurantToHomie,
) -> Result<(), AddHomiesRecentRestaurantError> {
    let add_recent_to_homie_params = AddRecentRestaurantToHomieParams::new(
        user_id.into(),
        homie_name.try_into()?,
        restaurant_name.try_into()?,
    );

    db.add_recent_restaurant_for_homie(&add_recent_to_homie_params)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return AddHomiesRecentRestaurantError::HomieAlreadyHasRecent {
                        name: add_recent_to_homie_params.name.as_str().to_string(),
                        restaurant_name: add_recent_to_homie_params
                            .restaurant_name
                            .as_str()
                            .to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return AddHomiesRecentRestaurantError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                AddHomiesRecentRestaurantError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            sqlx::Error::RowNotFound => AddHomiesRecentRestaurantError::NoRecentAdded,
            _ => AddHomiesRecentRestaurantError::UnknownDbError(e),
        })?;

    Ok(())
}

#[tracing::instrument(skip(db))]
pub async fn add_recent_restaurant_for_homies(
    homie_ids: Vec<impl Into<&HomieId> + Debug>,
    restaurant_id: impl Into<RestaurantId> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddRecentRestaurantToHomie,
) -> Result<(), AddHomiesRecentRestaurantError> {
    let add_recent_to_homies_params = AddRecentRestaurantToHomiesParams::new(
        user_id.into(),
        homie_ids.into_iter().map(|id| id.into()).collect(),
        restaurant_id.into(),
    );

    db.add_recent_restaurant_for_homies(&add_recent_to_homies_params)
        .await?;

    Ok(())
}

#[derive(Debug)]
struct AddRecentRestaurantToHomiesParams<'a> {
    user_id: UserId,
    homies_ids: Vec<&'a HomieId>,
    restaurant_id: RestaurantId,
}

impl<'a> AddRecentRestaurantToHomiesParams<'a> {
    fn new(user_id: UserId, homies_ids: Vec<&'a HomieId>, restaurant_id: RestaurantId) -> Self {
        Self {
            user_id,
            homies_ids,
            restaurant_id,
        }
    }
}

#[derive(Debug)]
struct AddRecentRestaurantToHomieParams {
    user_id: UserId,
    name: HomiesName,
    restaurant_name: RestaurantName,
}

impl AddRecentRestaurantToHomieParams {
    fn new(user_id: UserId, name: HomiesName, restaurant_name: RestaurantName) -> Self {
        Self {
            user_id,
            name,
            restaurant_name,
        }
    }
}

#[derive(Error, Debug)]
pub enum AddHomiesRecentRestaurantError {
    #[error(transparent)]
    HomieNameValidationError(#[from] HomieNameValidationError),

    #[error(transparent)]
    RestaurantNameValidationError(#[from] RestaurantNameValidationError),

    #[error("No recent added")]
    NoRecentAdded,

    #[error("Invalid User")]
    ForeignKeyViolation { constraint: String },

    #[error("{:?} already has {:?} recentd", name, restaurant_name)]
    HomieAlreadyHasRecent {
        name: String,
        restaurant_name: String,
    },

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

trait AddRecentRestaurantToHomie {
    async fn add_recent_restaurant_for_homie<'a>(
        &self,
        params: &AddRecentRestaurantToHomieParams,
    ) -> Result<(), sqlx::Error>;

    async fn add_recent_restaurant_for_homies<'a>(
        &self,
        params: &'a AddRecentRestaurantToHomiesParams<'a>,
    ) -> Result<(), sqlx::Error>;
}

impl AddRecentRestaurantToHomie for Pool<Postgres> {
    #[tracing::instrument(skip(self))]
    async fn add_recent_restaurant_for_homie<'a>(
        &self,
        params: &AddRecentRestaurantToHomieParams,
    ) -> Result<(), sqlx::Error> {
        let result = sqlx::query!(
            r#"
                insert into recent_restaurants (homie_id, user_id, restaurant_id)
                select 
                    h.id, 
                    $1,
                    r.id
                from homies h
                join restaurants r on r.name = $3 and r.user_id = $1
                where h.name = $2 and h.user_id = $1
                limit 1
                returning *;
            "#,
            params.user_id.as_i32(),
            params.name.as_str(),
            params.restaurant_name.as_str()
        )
        .fetch_one(self)
        .instrument(tracing::info_span!(
            "Adding recent restaurant to homie db query"
        ))
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn add_recent_restaurant_for_homies<'a>(
        &self,
        params: &'a AddRecentRestaurantToHomiesParams<'a>,
    ) -> Result<(), sqlx::Error> {
        let homie_ids: Vec<i32> = params.homies_ids.iter().map(|x| x.as_i32()).collect();

        sqlx::query!(
            r#"
with home_homies AS (select *
                     from UNNEST($1::integer[]) as homie_id)
insert
into recent_restaurants (homie_id, user_id, restaurant_id)
select h.id,
       $2,
       r.id
from home_homies hh
         join homies h on h.id = hh.homie_id
         join restaurants r on r.id = $3;
            "#,
            &homie_ids,
            params.user_id.as_i32(),
            params.restaurant_id.as_i32()
        )
        .fetch_all(self)
        .instrument(tracing::info_span!(
            "Adding recent restaurant to homie db query"
        ))
        .await?;
        Ok(())
    }
}
