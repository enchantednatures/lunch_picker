use anyhow::Result;
use lunch_picker::features::create_restaurant;
use sqlx::PgPool;

#[sqlx::test(fixtures("homies", "restaurants"))]
async fn test_add_existing_restaurant_fails(pool: PgPool) -> Result<()> {
    let actual = create_restaurant("Pizza".into(), -1, &pool).await;

    assert_eq!(
        format!("Restaurant already exists: {:?}", "Pizza"),
        actual.unwrap_err().to_string()
    );
    Ok(())
}

#[sqlx::test(fixtures("homies", "restaurants"))]
async fn test_add_restaurant(pool: PgPool) -> Result<()> {
    let result = create_restaurant("Thai".into(), -2, &pool).await?;

    assert_eq!("Thai", result.name);

    Ok(())
}
