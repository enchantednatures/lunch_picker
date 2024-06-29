use anyhow::Result;
use sqlx::Pool;
use sqlx::Postgres;
use std::fmt::Debug;

use crate::features::HomieId;
use crate::user::UserId;

use super::Restaurant;
use super::RestaurantRow;

#[tracing::instrument(skip(db))]
pub async fn get_candidate_restaurants(
    home_homies: &[&HomieId],
    user_id: impl Into<UserId> + Debug,
    db: &Pool<Postgres>,
) -> Result<Vec<Restaurant>> {
    let created_restaurant = db.get_candidates(home_homies, user_id.into()).await;

    Ok(created_restaurant)
}

trait GetCandidates {
    async fn get_candidates(&self, home_homies: &[&HomieId], user_id: UserId) -> Vec<Restaurant>;
}

impl GetCandidates for Pool<Postgres> {
    async fn get_candidates(&self, home_homies: &[&HomieId], user_id: UserId) -> Vec<Restaurant> {
        let candidates: Vec<RestaurantRow> = sqlx::query_as(
            r#"
with home_homies AS (select *
                     from UNNEST($1::integer[]) as homie_id),
     recents as (select restaurant_id, count(distinct homie_id) as occurrences
                 from homies_recents_restaurants_view v
                          join home_homies using (homie_id)
                        where v.user_id = $2
                 group by v.restaurant_id
                 order by occurrences desc),
     most_recents as (select restaurant_id
                      from recents
                      where occurrences = (select max(occurrences) from recents)),
     home_homies_favorites as (select r.id as restaurant_id, r.user_id as user_id, h.id as homie_id
                               from restaurants r
                                        join homies_favorite_restaurants hfr
                                             on r.user_id = hfr.user_id and r.id = hfr.restaurant_id
                                        join homies h on r.user_id = h.user_id and h.id = hfr.homie_id
                                        join home_homies hh on hh.homie_id = h.id
                               where r.user_id = 1
                                 and not exists (select 1
                                                 from homies_recents_restaurants_view v
                                                 where v.homie_id = h.id
                                                   and v.restaurant_id = r.id
                                                   and v.user_id = r.user_id))

select r.*
from (select *
      from (select restaurant_id, count(distinct homie_id) as occurrences
            from home_homies_favorites
            group by restaurant_id
--       having occurrences > max(select occurrences from t)
            order by occurrences desc) as t
      where not exists(select 1
                       from most_recents
                       where t.restaurant_id = restaurant_id)

      order by t.occurrences * random() desc
      limit 25) t
         join restaurants r on t.restaurant_id = r.id
"#,
        )
        .bind(&home_homies.iter().map(|h| h.as_i32()).collect::<Vec<i32>>())
            .bind(user_id.as_i32())
        .fetch_all(self)
        .await
        .unwrap();
        // todo stream rows
        candidates.into_iter().map(|r| r.into()).collect()
    }
}
