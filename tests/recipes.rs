use anyhow::Result;
use lunch_picker::features::create_homie;
use sqlx::PgPool;

#[cfg_attr(not(feature = "postgres_tests"), ignore)]
#[sqlx::test(fixtures("homies"))]
async fn test_add_existing_homie_fails(pool: PgPool) -> Result<()> {
    let actual = create_homie("Alice".to_string(), -1, &pool).await;

    assert_eq!(
        format!("Homie already exists: {:?}", "Alice"),
        actual.unwrap_err().to_string()
    );
    Ok(())
}

#[cfg_attr(not(feature = "postgres_tests"), ignore)]
#[sqlx::test(fixtures("homies"))]
async fn test_add_homie(pool: PgPool) -> Result<()> {
    let result = create_homie("Markus".to_string(), -1, &pool).await?;

    assert_eq!("Markus", result.name.as_str());

    Ok(())
}
