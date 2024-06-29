use anyhow::Result;
use sqlx::PgPool;

#[sqlx::test(fixtures(
    "homies",
    "restaurants",
    "homies_favorite_restaurants",
    "recent_restaurants"
))]
async fn test_restaurant_candidates(pool: PgPool) -> Result<()> {
    let home_homies = [-1, -2, -3];

    Ok(())
}
