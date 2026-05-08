use std::fmt::Debug;

use sqlx::Pool;
use sqlx::Sqlite;
use thiserror::Error;
use tracing::event;
use tracing::Instrument;
use tracing::Level;

use crate::features::HomieId;
use crate::features::RecipeId;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn add_recent_recipe_for_homies<'a, T, Y>(
    homie_ids: T,
    recipe_id: impl Into<RecipeId> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl AddRecentRecipeToHomies,
) -> Result<(), AddHomiesRecentRecipeError>
where
    T: IntoIterator<Item = Y> + Debug,
    Y: Into<HomieId> + Debug,
{
    let recipe_id = recipe_id.into();
    let homie_ids: Vec<HomieId> = homie_ids.into_iter().map(|id| id.into()).collect();

    let h: Vec<_> = homie_ids.iter().collect();
    let user_id = user_id.into();

    let params = AddRecentRecipeToHomiesParams::new(&user_id, h.as_slice(), &recipe_id);

    db.add_recent_recipe_for_homies(&params).await?;

    event!(
        Level::INFO,
        name = "Recent recipe added for home homies",
        homie_ids = ?&homie_ids,
        recipe_id = &recipe_id.as_i32()
    );

    Ok(())
}

#[derive(Debug)]
struct AddRecentRecipeToHomiesParams<'a> {
    user_id: &'a UserId,
    homies_ids: &'a [&'a HomieId],
    recipe_id: &'a RecipeId,
}

impl<'a> AddRecentRecipeToHomiesParams<'a> {
    fn new(
        user_id: &'a UserId,
        homies_ids: &'a [&'a HomieId],
        recipe_id: &'a RecipeId,
    ) -> Self {
        Self {
            user_id,
            homies_ids,
            recipe_id,
        }
    }
}

#[derive(Error, Debug)]
pub enum AddHomiesRecentRecipeError {
    #[error("No recent added")]
    NoRecentAdded,

    #[error("Invalid User")]
    ForeignKeyViolation { constraint: String },

    #[error("Unknown db error")]
    UnknownDbError(#[from] sqlx::Error),

    #[error("Unknown error")]
    Unknown,
}

pub(crate) trait AddRecentRecipeToHomies {
    async fn add_recent_recipe_for_homies<'a>(
        &self,
        params: &'a AddRecentRecipeToHomiesParams<'a>,
    ) -> Result<(), sqlx::Error>;
}

impl AddRecentRecipeToHomies for Pool<Sqlite> {
    #[tracing::instrument(skip(self))]
    async fn add_recent_recipe_for_homies<'a>(
        &self,
        params: &'a AddRecentRecipeToHomiesParams<'a>,
    ) -> Result<(), sqlx::Error> {
        let user_id = params.user_id.as_i32();
        let recipe_id = params.recipe_id.as_i32();
        let homie_ids: Vec<i32> = params.homies_ids.iter().map(|x| x.as_i32()).collect();

        let homie_ids = serde_json::to_string(&homie_ids)
            .expect("unable to serialize list of home homie ids as json");

        _ = sqlx::query(
            r#"
with home_homies AS (SELECT value as homie_id FROM json_each(?))
insert into recent_recipes (homie_id, user_id, recipe_id)
select h.id, ?, r.id
from home_homies hh
join homies h on h.id = hh.homie_id
join recipes r on r.id = ?;
            "#,
        )
        .bind(homie_ids)
        .bind(user_id)
        .bind(recipe_id)
        .execute(self)
        .instrument(tracing::info_span!(
            "Adding recent recipe to homies db query"
        ))
        .await?;

        Ok(())
    }
}
