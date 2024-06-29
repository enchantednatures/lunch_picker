use anyhow::Result;
use lunch_picker::features::get_candidate_restaurants;
use sqlx::PgPool;

#[sqlx::test(fixtures(
    "homies",
    "restaurants",
    "homies_favorite_restaurants",
    "recent_restaurants"
))]
async fn test_restaurant_candidates(pool: PgPool) -> Result<()> {
    let home_homies: Vec<_> = vec![-1, -2];

    let actual = get_candidate_restaurants(home_homies, 1, &pool).await?;

    assert_eq!(0, actual.len());

    Ok(())
}

