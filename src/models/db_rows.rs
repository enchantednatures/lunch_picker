use chrono::DateTime;
use chrono::Utc;

#[derive(Debug)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct RecentMeal {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

