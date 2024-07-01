use anyhow::Result;
use models::UserId;

use std::fmt::Debug;
use tracing::event;
use tracing::Instrument;


use crate::HomieId;

use super::Restaurant;
use super::RestaurantRow;

#[tracing::instrument(skip(db))]
pub async fn get_candidate_restaurants<'a, T, Y>(
    homie_ids: T,
    user_id: impl Into<UserId> + Debug,
    db: &impl GetCandidates,
) -> Result<Vec<Restaurant>>
where
    T: IntoIterator<Item = Y> + Debug,
    Y: Into<HomieId> + Debug,
{
    let homie_ids: Vec<HomieId> = homie_ids.into_iter().map(|id| id.into()).collect();

    let h: Vec<_> = homie_ids.iter().collect();
    let user_id = user_id.into();

    let created_restaurant = db.get_candidates(h.as_slice(), user_id).await;

    event!(
        tracing::Level::INFO,
        "Got candidates restaurants for homies"
    );

    Ok(created_restaurant)
}

trait GetCandidates {
    async fn get_candidates(&self, home_homies: &[&HomieId], user_id: UserId) -> Vec<Restaurant>;
}

