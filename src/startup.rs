use sqlx::SqlitePool;

pub struct AppState {
    pub db_pool: SqlitePool,
}

impl AppState {
    pub fn new(db_pool: SqlitePool) -> Self {
        AppState { db_pool }
    }
}
