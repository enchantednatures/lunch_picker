#![cfg(feature = "sqlite_tests")]

use anyhow::Result;
use lunch_picker::features::get_candidate_restaurants;

use sqlx::SqlitePool;

#[cfg_attr(not(feature = "sqlite_tests"), ignore)]
#[sqlx::test(fixtures(
    "homies",
    "restaurants",
    "homies_favorite_restaurants",
    "recent_restaurants"
))]
async fn test_restaurant_candidates(pool: SqlitePool) -> Result<()> {
    let home_homies: Vec<_> = vec![-1, -2];

    let actual = get_candidate_restaurants(home_homies, 1, &pool).await?;

    assert_eq!(0, actual.len());

    Ok(())
}
