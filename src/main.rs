use dialoguer::MultiSelect;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Uuid;

pub use models::db_rows::RecentMeal;
pub use models::db_rows::Recipe;

mod models;

struct Homie {
    pub id: Uuid,
    pub name: String,
}

async fn add_homie(db_pool: &PgPool, name: String) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO homies (name)
        VALUES ($1)
        "#,
        name
    )
        .execute(db_pool)
        .await?;
    Ok(())
}

async fn get_homies(db_pool: &PgPool) -> Result<Vec<Homie>, sqlx::Error> {
    let homies = sqlx::query_as!(
        Homie,
        r#"
        SELECT id, name
        FROM homies
        "#
    )
        .fetch_all(db_pool)
        .await?;
    Ok(homies)
}

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
    )
        .fetch_all(db_pool)
        .await?;
    Ok(recent_meals)
}

async fn get_home_homies(homies: &Vec<Homie>) -> Vec<String> {
    let homies_names = homies
        .iter()
        .map(|h| h.name.as_str())
        .collect::<Vec<&str>>();
    let chosen = MultiSelect::new()
        .with_prompt("Who's home?")
        .items(&homies_names)
        .interact()
        .unwrap();
    if chosen.is_empty() {
        println!("No homies selected");
        return vec![];
    } else {
        println!("Homies selected: {:?}", chosen);
    }
    let home_homies = chosen
        .iter()
        .map(|&index| {
            homies_names[index].to_string()
        })
        .collect();
    home_homies
}

async fn get_recipes(_db_pool: &PgPool) -> Result<Vec<Recipe>, sqlx::Error> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_url = get_db_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    let homies = get_homies(&pool).await?;

    let _home_homies = get_home_homies(&homies).await;
    let recent = get_recent_meals(&pool).await?;
    println!("recents: {:?}", recent);
    Ok(())
}
