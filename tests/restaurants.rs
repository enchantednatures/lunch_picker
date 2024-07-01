#![cfg(feature = "sqlite_tests")]

use anyhow::Result;
use lunch_picker::features::create_restaurant;
use sqlx::SqlitePool;

#[cfg_attr(not(feature = "sqlite_tests"), ignore)]
#[sqlx::test( fixtures("homies", "restaurants"))]
async fn test_add_existing_restaurant_fails(pool: SqlitePool) -> Result<()> {
    let actual = create_restaurant("Pizza".into(), -1, &pool).await;

    assert_eq!(
        format!("Restaurant already exists: {:?}", "Pizza"),
        actual.unwrap_err().to_string()
    );
    Ok(())
}

#[cfg_attr(not(feature = "sqlite_tests"), ignore)]
#[sqlx::test( fixtures("homies", "restaurants"))]
async fn test_add_restaurant(pool: SqlitePool) -> Result<()> {
    let result = create_restaurant("Thai".into(), -2, &pool).await?;

    assert_eq!("Thai", result.name.as_str());

    Ok(())
}
