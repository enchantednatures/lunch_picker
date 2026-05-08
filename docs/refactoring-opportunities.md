# Refactoring Opportunities

This document catalogs specific, actionable refactorings organized by theme. Each includes the current state, the desired state, and the rationale.

---

## 1. Consolidate Error Handling

### Current State
Three different error styles coexist:
- `thiserror` derive macros (majority)
- Manual `Display` impl (`features/recipes/create.rs`)
- `anyhow` at application boundaries

### Desired State
Standardize on `thiserror` for domain errors and `anyhow` for top-level propagation.

### Action
1. Rewrite `CreateRecipeError` to use `#[derive(Error)]`:

```rust
// Before (features/recipes/create.rs)
#[derive(Debug)]
pub enum CreateRecipeError {
    Unknown,
    UnknownDbError(String),
    InvalidName,
    ForeignKeyViolation { constraint: String },
    RecipeAlreadyExists,
}

impl Display for CreateRecipeError { ... }
impl Error for CreateRecipeError {}

// After
#[derive(Error, Debug)]
pub enum CreateRecipeError {
    #[error("Unknown error")]
    Unknown,
    #[error("Unknown db error: {0}")]
    UnknownDbError(String),
    #[error("Invalid name")]
    InvalidName,
    #[error("Foreign key violation: {constraint}")]
    ForeignKeyViolation { constraint: String },
    #[error("Recipe already exists")]
    RecipeAlreadyExists,
}
```

2. Ensure all feature errors derive from `thiserror::Error`.

---

## 2. Remove Unused Traits and Simplify Data Layer

### Current State
Every DB operation has a custom trait, but all are implemented only for `Pool<Sqlite>`:

```rust
pub trait CreateHomie {
    async fn create_homie<'a>(
        &self,
        params: CreateHomieParams<'a>,
    ) -> Result<Homie, CreateHomieError>;
}

impl CreateHomie for Pool<Sqlite> { ... }
```

### Desired State
For a CLI of this size, plain async functions accepting `&Pool<Sqlite>` are sufficient. If testability is needed later, introduce a single `Repository` struct.

### Action
Replace the trait-per-operation pattern with plain functions:

```rust
// Before
pub async fn create_homie(
    homie_name: impl TryInto<HomiesName> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateHomie,
) -> Result<Homie, CreateHomieError> { ... }

// After
pub async fn create_homie(
    homie_name: impl TryInto<HomiesName> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &Pool<Sqlite>,
) -> Result<Homie, CreateHomieError> { ... }
```

This eliminates:
- 10+ trait definitions
- 10+ params structs
- Complex generic bounds in `interaction.rs`
- `private_interfaces` compiler warnings

If mocking is needed for tests, consider:

```rust
pub struct Repository<'a>(pub &'a Pool<Sqlite>);

impl<'a> Repository<'a> {
    pub async fn create_homie(&self, ...) -> Result<Homie, CreateHomieError> { ... }
}
```

---

## 3. Fix Panics and Unwraps

### Current State

**`src/interaction.rs`**:
```rust
pub async fn get_home_homies(homies: &[Homie]) -> Result<Vec<&Homie>> {
    if homies.is_empty() {
        tracing::error!("No homies found");
        panic!(); // ❌
    }
    ...
}
```

**`src/features/restaurants/get_candidates.rs`**:
```rust
.fetch_all(self)
.await
.unwrap(); // ❌
```

### Desired State
Return errors properly.

### Action

```rust
// interaction.rs
#[derive(Error, Debug)]
pub enum InteractionError {
    #[error("No homies found. Please add homies first.")]
    NoHomies,
}

pub async fn get_home_homies(homies: &[Homie]) -> Result<Vec<&Homie>> {
    if homies.is_empty() {
        return Err(InteractionError::NoHomies.into());
    }
    ...
}
```

```rust
// get_candidates.rs
pub async fn get_candidate_restaurants(...) -> Result<Vec<Restaurant>, GetCandidatesError> {
    ...
    let candidates: Vec<RestaurantRow> = sqlx::query_as(...)
        .bind(...)
        .fetch_all(self)
        .await?; // ✅ Propagate error
    ...
}

#[derive(Error, Debug)]
pub enum GetCandidatesError {
    #[error("Database error")]
    Db(#[from] sqlx::Error),
}
```

---

## 4. Eliminate Hardcoded `user_id = 1`

### Current State
`src/main.rs`:
```rust
const CLI_USER_ID: i32 = 1;
```

And in SQL (`get_candidates.rs`):
```sql
where r.user_id = 1  -- ❌
```

### Desired State
Support configurable user IDs.

### Action
1. Add `user_id` to `Settings`:
```rust
pub struct Settings {
    pub database_url: String,
    pub telemetry_enabled: bool,
    pub user_id: i32,
}
```

2. Accept `--user-id` CLI flag or read from config.

3. Fix SQL:
```sql
where r.user_id = ?  -- ✅ Use bound parameter
```

---

## 5. Fix Risky `Drop` Implementation

### Current State
```rust
impl Drop for AppState {
    fn drop(&mut self) {
        futures::executor::block_on(self.db.close());
        opentelemetry::global::shutdown_tracer_provider();
    }
}
```

### Desired State
Explicit cleanup before exit, no `block_on` in `Drop`.

### Action

```rust
// Remove Drop impl entirely

// In main(), before Ok(()):
app_state.db.close().await;
opentelemetry::global::shutdown_tracer_provider();
```

Also, remove the `futures` dependency or pin it to a specific version. If `block_on` is still needed elsewhere, use `tokio::runtime::Handle::try_current().block_on(...)` with care.

---

## 6. Add Missing Validation

### Current State
`create_restaurant` accepts any `String`:
```rust
pub async fn create_restaurant(
    restaurant_name: String,  // ❌ No validation
    user_id: impl Into<UserId> + Debug,
    db: &impl CreateRestaurant,
) -> Result<Restaurant, CreateRestaurantError> { ... }
```

### Desired State
Apply `RestaurantName` validation, matching `create_homie`.

### Action
```rust
pub async fn create_restaurant(
    restaurant_name: impl TryInto<RestaurantName, Error = RestaurantNameValidationError> + Debug,
    user_id: impl Into<UserId> + Debug,
    db: &Pool<Sqlite>,
) -> Result<Restaurant, CreateRestaurantError> {
    let name: RestaurantName = restaurant_name.try_into()?;
    let restaurant = CreateRestaurantParams::new(user_id.into(), name.as_str());
    ...
}
```

Update `CreateRestaurantError`:
```rust
#[derive(Error, Debug)]
pub enum CreateRestaurantError {
    #[error(transparent)]
    ValidationError(#[from] RestaurantNameValidationError),
    ...
}
```

---

## 7. Simplify Name-Based Queries to ID-Based

### Current State
Favorites and recents are looked up by name:
```sql
join restaurants r on r.name = ? and r.user_id = ?
where h.name = ? and h.user_id = ?
```

### Desired State
CLI commands resolve names to IDs once, then internal APIs use IDs.

### Action
1. Add a name-to-ID resolution function:
```rust
pub async fn get_homie_by_name(
    user_id: UserId,
    name: &str,
    db: &Pool<Sqlite>,
) -> Result<Homie, sqlx::Error> { ... }
```

2. Update CLI handlers to resolve names before calling internal functions:
```rust
// main.rs
Command::Homies(homie_command) => match homie_command {
    Homies::Restaurants(AddRestaurant::Add { homie_name, restaurant_name }) => {
        let homie = get_homie_by_name(CLI_USER_ID, &homie_name, &app_state.db).await?;
        let restaurant = get_restaurant_by_name(CLI_USER_ID, &restaurant_name, &app_state.db).await?;
        add_homies_favorite_restaurant(homie.id, restaurant.id, CLI_USER_ID, &app_state.db).await?;
    }
}
```

3. Simplify SQL to use IDs directly:
```sql
insert into homies_favorite_restaurants (homie_id, user_id, restaurant_id)
values (?, ?, ?);
```

---

## 8. Remove Dead Code

### Action Items

| Item | Action |
|------|--------|
| `src/features/recipes/` | Delete or fully implement. Currently dead weight. |
| `src/features/get_homie_by_name.rs` | Complete implementation or delete. |
| `src/features.rs` lines 7–10 | Remove empty module declarations. |
| `src/main.rs` lines 47–50 | Remove commented `HomiePaging` trait. |
| `src/main.rs` lines 274–275 | Remove commented cleanup code. |
| `migrations/f/20240628044251_recipes.up.sql.sql` | Rename or delete. |

---

## 9. Use Transactions for Multi-Row Operations

### Current State
`add_recent_restaurant_for_homies` inserts multiple rows without a transaction:
```rust
sqlx::query!(...).execute(self).await?;
```

### Desired State
Wrap in a transaction and verify row count.

### Action
```rust
pub async fn add_recent_restaurant_for_homies(...) -> Result<(), AddHomiesRecentRestaurantError> {
    let mut tx = self.db.begin().await?;
    
    let result = sqlx::query!(...)
        .execute(&mut *tx)
        .await?;
    
    if result.rows_affected() != homie_ids.len() as u64 {
        tx.rollback().await?;
        return Err(AddHomiesRecentRestaurantError::PartialInsert);
    }
    
    tx.commit().await?;
    Ok(())
}
```

---

## 10. Fix Config Path Conventions

### Current State
- Config: `~/.config/local/lunch.json`
- DB: `~/.local/state/lunch.db`

### Desired State
Follow XDG Base Directory spec with app-named directories:
- Config: `~/.config/lunch_picker/config.json`
- Data: `~/.local/share/lunch_picker/lunch.db`

### Action
```rust
let mut config_dir = dirs::config_dir().expect("No config dir");
config_dir.push("lunch_picker");
config_dir.push("config.json");

let mut data_dir = dirs::data_dir().expect("No data dir");
data_dir.push("lunch_picker");
// Use data_dir for DB location
```

Provide a migration path: if old config exists, read it and rewrite to new location.

---

## 11. Introduce a Single Repository Struct

If keeping some abstraction, a single `Repository` struct reduces boilerplate:

```rust
pub struct Repository<'a>(pub &'a Pool<Sqlite>);

impl<'a> Repository<'a> {
    pub async fn create_homie(&self, name: &HomiesName, user_id: UserId) -> Result<Homie, CreateHomieError> { ... }
    pub async fn get_all_homies(&self, user_id: UserId) -> Result<Vec<Homie>, sqlx::Error> { ... }
    pub async fn create_restaurant(&self, name: &RestaurantName, user_id: UserId) -> Result<Restaurant, CreateRestaurantError> { ... }
    // ... etc
}
```

This removes 10+ traits and their associated params structs while keeping DB logic centralized.

---

## 12. Apply Auto-Fixable Clippy Lints

Run:
```bash
cargo clippy --fix --lib -p lunch_picker
cargo clippy --fix --bin "lunch_picker" -p lunch_picker
```

This will fix:
- needless returns in `interaction.rs`
- unused lifetimes in trait methods
- lifetime elision warnings on `as_view()`
- needless borrows in `get_candidates.rs`

Manual fixes needed for:
- `private_interfaces` warnings (make params structs `pub(crate)` or redesign)
- `dead_code` in `get_homie_by_name.rs`

---

## 13. Improve Testability

### Current State
Tests require `sqlite_tests` feature and use `sqlx::test` with fixtures.

### Desired State
Unit tests for pure logic (validation, candidate ranking) without DB dependency.

### Action
1. Extract the candidate ranking/selection logic from SQL into Rust:
```rust
fn select_candidates(
    all_favorites: &[(HomieId, RestaurantId)],
    recent_picks: &[(HomieId, RestaurantId)],
) -> Vec<RestaurantId> { ... }
```

2. Test this function with in-memory data.

3. Keep integration tests for SQL correctness, but add unit tests for business rules.

---

## 14. Add Missing Indexes

### Action
Add migration:
```sql
create index idx_recent_restaurants_user_homie_date 
    on recent_restaurants(user_id, homie_id, date);

create index idx_homies_fav_restaurants_user_homie 
    on homies_favorite_restaurants(user_id, homie_id);
```

Drop redundant indexes:
```sql
drop index homies_user_uindex;
drop index restaurant_user_uindex;
```

---

## Priority Matrix

| Refactoring | Effort | Impact | Priority |
|-------------|--------|--------|----------|
| Fix panics/unwraps | Low | Critical | P0 |
| Fix hardcoded user_id | Low | High | P0 |
| Fix Drop impl | Low | High | P0 |
| Add validation to create_restaurant | Low | Medium | P1 |
| Auto-fix clippy lints | Low | Low | P1 |
| Remove dead code | Low | Low | P1 |
| Simplify traits → plain fns | Medium | High | P1 |
| Fix config paths | Medium | Low | P2 |
| Name → ID resolution | Medium | Medium | P2 |
| Add transactions | Medium | Medium | P2 |
| Extract unit-testable logic | High | Medium | P2 |
| Add missing indexes | Low | Medium | P2 |
