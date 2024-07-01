use std::fmt::Debug;

use features::CreateHomie;
use features::CreateHomieError;
use models::UserId;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use super::models::Homie;
use super::HomieNameValidationError;
use super::HomieRow;
use super::HomiesName;


impl CreateHomie for Pool<Sqlite> {
    #[tracing::instrument(skip(self, params))]
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieError> {
        let homie: HomieRow = sqlx::query_as(
            r#"INSERT INTO homies (user_id, name) VALUES (?, ?) RETURNING id, user_id, name"#,
        )
        .bind(params.user_id)
        .bind(params.name.to_string())
        // .bind((params.user_id, params.name.to_string()))
        .fetch_one(self)
        .instrument(tracing::info_span!("Insert Homie Query"))
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_error) => {
                if db_error.is_unique_violation() {
                    return CreateHomieError::HomieAlreadyExists {
                        name: params.name.to_string(),
                    };
                } else if db_error.is_foreign_key_violation() {
                    return CreateHomieError::ForeignKeyViolation {
                        constraint: db_error
                            .constraint()
                            .expect("Constraint should be named if it is a ForeignKeyViolation")
                            .to_string(),
                    };
                }
                CreateHomieError::UnknownDbError(sqlx::Error::Database(db_error))
            }
            _ => CreateHomieError::UnknownDbError(e),
        })?;
        Ok(homie.into())
    }
}
