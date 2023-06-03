use chrono::DateTime;
use chrono::Utc;
use sqlx::types::Uuid;

#[derive(Debug)]
pub struct Recipe {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct RecentMeal {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
