use std::fmt::Debug;

use features::GetAllHomies;
use sqlx::Pool;

use sqlx::Sqlite;
use thiserror::Error;
use tracing::Instrument;

use models::UserId;

use super::Homie;
use super::HomieRow;

impl GetAllHomies for Pool<Sqlite> {
    #[tracing::instrument(name = "Getting all Homies", skip(self))]
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error> {
        let homie: Vec<HomieRow> =
            sqlx::query_as(r#"select id, user_id, name from homies where user_id = ?"#)
                .bind(params.as_i32())
                .fetch_all(self)
                .instrument(tracing::info_span!("Querying all homies"))
                .await?;
        Ok(homie.into_iter().map(|x| x.into()).collect())
    }
}
