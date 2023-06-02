mod models;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub use models::db_rows::RecentMeal;
pub use models::db_rows::Recipe;

fn get_db_url() -> String {
    dotenv::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

async fn get_recent_meals(db_pool: &PgPool) -> Result<Vec<RecentMeal>, sqlx::Error> {
    let recent_meals = sqlx::query_as!(
       RecentMeal,
       r#"
       SELECT id, name, created_at
       FROM recent_meals
       ORDER BY created_at DESC
       LIMIT 5
       "#
   ).fetch_all(db_pool).await?;
    return Ok(recent_meals);
}

async fn get_recipes(db_pool: &PgPool) -> Result<Vec<Recipe>, sqlx::Error> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_url = get_db_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let recents = get_recent_meals(&pool).await?;
    println!("recents: {:?}", recents);
    Ok(())
}
