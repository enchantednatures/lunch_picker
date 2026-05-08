use anyhow::Result;
use sqlx::Pool;
use sqlx::Sqlite;
use std::fmt::Debug;
use tracing::event;
use tracing::Instrument;

use crate::features::HomieId;
use crate::features::Recipe;
use crate::features::RecipeRow;
use crate::user::UserId;

#[tracing::instrument(skip(db))]
pub async fn get_candidate_recipes<'a, T, Y>(
    homie_ids: T,
    user_id: impl Into<UserId> + Debug,
    db: &impl GetRecipeCandidates,
) -> Result<Vec<Recipe>>
where
    T: IntoIterator<Item = Y> + Debug,
    Y: Into<HomieId> + Debug,
{
    let homie_ids: Vec<HomieId> = homie_ids.into_iter().map(|id| id.into()).collect();
    let h: Vec<_> = homie_ids.iter().collect();
    let user_id = user_id.into();

    let candidates = db.get_recipe_candidates(h.as_slice(), user_id).await?;

    event!(tracing::Level::INFO, "Got candidate recipes for homies");
    Ok(candidates.into_iter().map(|r| r.into()).collect())
}

pub(crate) trait GetRecipeCandidates {
    async fn get_recipe_candidates(
        &self,
        home_homies: &[&HomieId],
        user_id: UserId,
    ) -> Result<Vec<RecipeRow>, sqlx::Error>;
}

impl GetRecipeCandidates for Pool<Sqlite> {
    async fn get_recipe_candidates(
        &self,
        home_homies: &[&HomieId],
        user_id: UserId,
    ) -> Result<Vec<RecipeRow>, sqlx::Error> {
        let candidates: Vec<RecipeRow> = sqlx::query_as(
            r#"
with home_homies AS (SELECT value as homie_id FROM json_each(?)),
     recents as (select recipe_id, count(distinct homie_id) as occurrences
                 from (select
                           recipe_id,
                           homie_id,
                           user_id,
                           date,
                           rank() over (partition by homie_id order by date desc) as rank
                       from recent_recipes) as v
                 where v.rank <= 5
                   and v.date > current_date - '21 days'
                   and v.user_id = ?
                 join home_homies using (homie_id)
                 group by v.recipe_id
                 order by occurrences desc),
     most_recents as (select recipe_id
                      from recents
                      where occurrences = (select max(occurrences) from recents)),
     home_homies_favorites as (select r.id as recipe_id, r.user_id as user_id, h.id as homie_id
                               from recipes r
                                        join homies_favorite_recipes hfr
                                             on r.user_id = hfr.user_id and r.id = hfr.recipe_id
                                        join homies h on r.user_id = h.user_id and h.id = hfr.homie_id
                                        join home_homies hh on hh.homie_id = h.id
                               where r.user_id = ?
                                 and not exists (select 1
                                                 from (select recipe_id, homie_id, date,
                                                              rank() over (partition by homie_id order by date desc) as rank
                                                       from recent_recipes
                                                       where user_id = ?) v
                                                 where v.recipe_id = r.id
                                                   and v.rank <= 5
                                                   and v.date > current_date - '21 days'
                                                   and v.homie_id = h.id))
select r.*
from (select *
      from (select recipe_id, count(distinct homie_id) as occurrences
            from home_homies_favorites
            group by recipe_id
            order by occurrences desc) as t
      where not exists(select 1
                       from most_recents
                       where t.recipe_id = recipe_id)
      order by t.occurrences * random() desc
      limit 25) t
         join recipes r on t.recipe_id = r.id
            "#,
        )
        .bind(serde_json::to_string(&home_homies.iter().map(|h| h.as_i32()).collect::<Vec<i32>>()).expect("unable to serialize list of home homie ids as json"))
        .bind(user_id.as_i32())
        .bind(user_id.as_i32())
        .bind(user_id.as_i32())
        .fetch_all(self)
        .instrument(tracing::info_span!("Getting candidate recipes for homies", { "count of home homies" } = home_homies.len()) )
        .await?;

        Ok(candidates)
    }
}
