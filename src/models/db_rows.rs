#[derive(Debug)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    // pub created_at: NaiveDateTime,
    // pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct RecentMeal {
    pub id: i64,
    pub name: String,
    // pub created_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct Homie {
    pub id: i64,
    pub name: String,
}

#[derive(Debug)]
pub struct HomiesFavorite {
    pub id: i64,
    pub homie_id: i64,
    pub recipe_id: i64,
}
