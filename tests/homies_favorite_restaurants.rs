use anyhow::Result;
use lunch_picker::features::add_homies_favorite_restaurant;
use sqlx::PgPool;

#[sqlx::test(fixtures("homies", "restaurants", "homies_favorite_restaurants"))]
async fn duplicate_cannot_be_added(pool: PgPool) -> Result<()> {
    let actual =
        add_homies_favorite_restaurant("Alice".to_string(), "Pizza".to_string(), -1, &pool).await;

    assert_eq!(
        format!("{:?} already has {:?} favorited", "Alice", "Pizza"),
        actual.unwrap_err().to_string()
    );
    Ok(())
}

#[sqlx::test(fixtures("homies", "restaurants", "homies_favorite_restaurants"))]
async fn valid(pool: PgPool) -> Result<()> {
    Ok(add_homies_favorite_restaurant("Bob".to_string(), "Pizza".to_string(), -1, &pool).await?)
}

#[sqlx::test(fixtures("homies", "restaurants", "homies_favorite_restaurants"))]
async fn no_favorites_are_added_for_non_existant_homies(pool: PgPool) -> Result<()> {
    let actual =
        add_homies_favorite_restaurant("Bobberto".to_string(), "Pizza".to_string(), -1, &pool).await;

    assert_eq!("No favorite added", actual.unwrap_err().to_string());
    Ok(())
}
