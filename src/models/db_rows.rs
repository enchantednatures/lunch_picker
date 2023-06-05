use chrono::{NaiveDateTime};


#[derive(Debug)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct RecentMeal {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
}
