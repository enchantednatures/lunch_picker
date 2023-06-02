use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use dialoguer::MultiSelect;
pub use models::db_rows::RecentMeal;
pub use models::db_rows::Recipe;

mod models;

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

async fn get_home_homies() -> Vec<String> {
    let homies = vec!["Hunter".to_string(), "Sienna".to_string()];
    let chosen = MultiSelect::new()
        .with_prompt("Who's home?")
        .items(&homies)
        .interact()
        .unwrap();
    if chosen.is_empty() {
        println!("No homies selected");
        return vec![];
    } else {
        println!("Homies selected: {:?}", chosen);
    }
    let mut home_homies:Vec<String> = vec![];
    for i in chosen {
        home_homies.push(homies[i].clone().to_string());
    }
    return home_homies;
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

    let home_homies = get_home_homies().await;
    let recents = get_recent_meals(&pool).await?;
    println!("recents: {:?}", recents);
    Ok(())
}
