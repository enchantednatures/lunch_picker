#![cfg(feature = "sqlite_tests")]

use anyhow::Result;
use lunch_picker::features::add_recent_restaurant_for_homie;
use sqlx::SqlitePool;

#[cfg_attr(not(feature = "sqlite_tests"), ignore)]
#[sqlx::test(fixtures(
    "homies",
    "restaurants",
    "homies_favorite_restaurants",
    "recent_restaurants"
))]
async fn duplicate_cannot_be_added(pool: SqlitePool) -> Result<()> {
    let actual =
        add_recent_restaurant_for_homie("Alice".to_string(), "Pizza".to_string(), -1, &pool).await;

    assert_eq!(
        format!("{:?} already has {:?} recentd", "Alice", "Pizza"),
        actual.unwrap_err().to_string()
    );
    Ok(())
}

#[cfg_attr(not(feature = "sqlite_tests"), ignore)]
#[sqlx::test(fixtures(
    "homies",
    "restaurants",
    "homies_favorite_restaurants",
    "recent_restaurants"
))]
async fn valid(pool: SqlitePool) -> Result<()> {
    Ok(
        add_recent_restaurant_for_homie("Ringo".to_string(), "Pizza".to_string(), -1, &pool)
            .await?,
    )
}

#[cfg_attr(not(feature = "sqlite_tests"), ignore)]
#[sqlx::test(fixtures(
    "homies",
    "restaurants",
    "homies_favorite_restaurants",
    "recent_restaurants"
))]
async fn no_recents_are_added_for_non_existant_homies(pool: SqlitePool) -> Result<()> {
    let actual =
        add_recent_restaurant_for_homie("Bobbert".to_string(), "Pizza".to_string(), -1, &pool)
            .await;

    assert_eq!("No recent added", actual.unwrap_err().to_string());
    Ok(())
}
