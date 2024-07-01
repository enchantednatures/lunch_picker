use std::fmt::Debug;

use models::UserId;
use sqlx::Pool;

#[cfg(feature = "postgres")]
use sqlx::Postgres;
#[cfg(feature = "sqlite")]
use sqlx::Sqlite;
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
    let restaurant = CreateRestaurantParams::new(user_id.into(), &restaurant_name);

    let created_restaurant = db.create_restaurant(restaurant).await?;

    Ok(created_restaurant)
}

#[derive(Debug)]
struct CreateRestaurantParams<'a> {
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

#[cfg(feature = "postgres")]
impl CreateRestaurant for Pool<Postgres> {
    #[tracing::instrument(skip(self, params))]
    async fn create_restaurant<'a>(
        &self,
        params: CreateRestaurantParams<'a>,
    ) -> Result<Restaurant, CreateRestaurantError> {
        let restaurant: RestaurantRow = sqlx::query_as(
            r#"INSERT INTO restaurants (user_id, name) VALUES ($1, $2) RETURNING id, user_id, name"#,
        )
        .bind(params.user_id)
        .bind(params.name)
        .fetch_one(self)
        .instrument(tracing::info_span!("Insert Restaurant into Database"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateRestaurantError::RestaurantAlreadyExists {
                        name: params.name.to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return CreateRestaurantError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                CreateRestaurantError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateRestaurantError::UnknownDbError(e),
        })?;
        Ok(restaurant.into())
    }
}

#[cfg(feature = "sqlite")]
impl CreateRestaurant for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_restaurant<'a>(
        &self,
        params: CreateRestaurantParams<'a>,
    ) -> Result<Restaurant, CreateRestaurantError> {
        let restaurant: RestaurantRow = sqlx::query_as(
            r#"INSERT INTO restaurants (user_id, name) VALUES (?, ?) RETURNING id, user_id, name"#,
        )
        .bind(params.user_id)
        .bind(params.name)
        .fetch_one(self)
        .instrument(tracing::info_span!("Insert Restaurant into Database"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateRestaurantError::RestaurantAlreadyExists {
                        name: params.name.to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return CreateRestaurantError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                CreateRestaurantError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateRestaurantError::UnknownDbError(e),
        })?;
        Ok(restaurant.into())
    }
}
