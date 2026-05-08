# Database Layer Review

## Schema Design

### Tables

| Table | Purpose | Key Constraints |
|-------|---------|-----------------|
| `users` | Application users | `id` PK, seeded with `id = 1` |
| `homies` | People in the lunch group | `(user_id, name)` unique, FK to `users` |
| `restaurants` | Available restaurants | `(user_id, name)` unique, FK to `users` |
| `recent_restaurants` | History of picks | Composite PK `(homie_id, restaurant_id, date)` |
| `homies_favorite_restaurants` | Many-to-many favorites | Composite PK `(homie_id, restaurant_id)` |

### Views

| View | Purpose |
|------|---------|
| `homies_recents_restaurants_view` | Ranks recent restaurants per homie (top 5 within 21 days) |

### Schema Strengths

1. **Multi-tenancy ready**: Every table includes `user_id` with foreign keys and cascading deletes.
2. **CHECK constraints**: Names are validated at the DB level (`length(name) = length(trim(name)) and length(name) > 0`).
3. **Composite foreign keys**: `recent_restaurants` references `(restaurant_id, user_id)` and `(homie_id, user_id)`, ensuring cross-user isolation.
4. **Cascading deletes**: Deleting a user cleans up all related data automatically.
5. **Window functions**: The view uses `rank() over (partition by homie_id order by date desc)` for efficient recent exclusion.

### Schema Weaknesses

1. **No `ON UPDATE CASCADE`**: Only `ON DELETE CASCADE` is specified. If user IDs were mutable (they aren't, being synthetic), updates would fail.
2. **No timestamps on favorites**: `homies_favorite_restaurants` lacks `created_at`, making audit trails impossible.
3. **No soft deletes**: All deletes are physical. There's no way to recover accidentally deleted restaurants or homies.
4. **`users` table is sparse**: Only `id`, `created_at`, `updated_at`. No email, name, or auth fields — fine for a single-user CLI, but odd given the multi-tenancy schema.

---

## Query Analysis

### 1. `get_candidates` — Restaurant Selection Algorithm

**File**: `src/features/restaurants/get_candidates.rs`

This is the most complex and important query in the application. It determines which restaurants are valid options for lunch.

#### SQL Breakdown

```sql
with home_homies AS (SELECT value as homie_id FROM json_each(?)),
     recents as (...),
     most_recents as (...),
     home_homies_favorites as (...)
select r.*
from (...) t
join restaurants r on t.restaurant_id = r.id
```

#### Logic
1. Accept a JSON array of homie IDs via `json_each(?)`.
2. Find recent restaurants for those homies (ranked by frequency).
3. Identify the "most recent" restaurants (those with max occurrence count).
4. Find all restaurants that are favorites of ALL home homies and NOT in recent picks.
5. Exclude most-recents, then order by `occurrences * random() desc` and limit to 25.
6. Join back to `restaurants` for full row data.

#### Issues

**A. Hardcoded `user_id`**
```sql
where r.user_id = 1  -- ❌ Should be `r.user_id = ?`
```
This ignores the `user_id` parameter passed to the function, breaking multi-tenancy.

**B. Unnecessary subquery nesting**
The query has 4 CTEs and 3 levels of nested subqueries. While SQLite optimizes CTEs reasonably well, this complexity makes the query hard to reason about and maintain.

**C. `.unwrap()` on query execution**
```rust
.fetch_all(self)
.await
.unwrap();
```
Any DB error panics. Should return `Result<Vec<Restaurant>, sqlx::Error>`.

**D. JSON serialization for integer arrays**
```rust
.bind(&serde_json::to_string(&home_homies.iter().map(|h| h.as_i32()).collect::<Vec<i32>>())...)
```
Passing a JSON string just to use `json_each` is inefficient. SQLite supports binding multiple values via `IN (...)` or table-valued functions, but sqlx doesn't support dynamic `IN` clauses easily. Alternative: generate the query dynamically or use a temporary table.

**E. `random()` in ORDER BY with `LIMIT`**
```sql
order by t.occurrences * random() desc
limit 25
```
This weights randomness by how many homies favor the restaurant (more favored = higher weight). The `limit 25` then caps candidates before presenting to the user. This is reasonable but non-deterministic — two runs with the same data can produce different candidate lists.

---

### 2. `add_recent_restaurant_for_homies` — Batch Insert

**File**: `src/features/recents/add_recent_restaurant.rs`

```sql
with home_homies AS (SELECT value as homie_id FROM json_each(?))
insert into recent_restaurants (homie_id, user_id, restaurant_id)
select h.id, ?, r.id
from home_homies hh
join homies h on h.id = hh.homie_id
join restaurants r on r.id = ?;
```

#### Issues

**A. No transaction wrapping**
The `execute()` is not wrapped in a transaction. If the app crashes after partial execution, some homies will have the recent recorded and others won't.

**B. Ignores result**
```rust
_ = sqlx::query!(...).execute(self).await?;
```
The `execute` result contains `rows_affected()`, which should be validated against the expected number of homies.

**C. JSON serialization overhead**
Same `serde_json::to_string` pattern as `get_candidates`.

---

### 3. `add_homies_favorite_restaurant` — Insert by Name

**File**: `src/features/homies_favorites/restaurants.rs`

```sql
insert into homies_favorite_restaurants (homie_id, user_id, restaurant_id)
select h.id, ?, r.id
from homies h
join restaurants r on r.name = ? and r.user_id = ?
where h.name = ? and h.user_id = ?
limit 1
returning *;
```

#### Issues

**A. Name-based lookup instead of ID**
Joining on `name` is fragile. If two restaurants have the same name ( prevented by unique index, but still), this would be ambiguous. Using IDs would be safer and faster.

**B. `limit 1` compensates for poor design**
The `limit 1` is only needed because the API accepts names. With IDs, the query would be a clean single-row insert.

**C. `fetch_one` for insert**
Using `fetch_one` with `RETURNING *` when only success/failure matters adds unnecessary overhead.

---

### 4. `remove_homies_favorite_restaurant` — Delete by Name

**File**: `src/features/homies_favorites/remove_homies_favorite_restaurant.rs`

```sql
delete from homies_favorite_restaurants
where exists (
    select distinct 1
    from homies_favorite_restaurants f
    inner join homies h on h.name = ? and h.id = f.homie_id
    inner join restaurants r on r.name = ? and r.id = f.restaurant_id
    where f.user_id = ?
    and homies_favorite_restaurants.user_id = f.user_id
    and homies_favorite_restaurants.homie_id = f.homie_id
    and homies_favorite_restaurants.restaurant_id = f.restaurant_id
)
returning *;
```

#### Issues

**A. Overly complex query**
The `EXISTS` subquery with self-reference is unnecessarily complex. A direct DELETE with a subquery would suffice:

```sql
delete from homies_favorite_restaurants
where user_id = ?
  and homie_id = (select id from homies where name = ? and user_id = ?)
  and restaurant_id = (select id from restaurants where name = ? and user_id = ?);
```

**B. `fetch_one` for DELETE**
Same issue as above — using `fetch_one` implies expecting exactly one row, but the operation conceptually allows 0 or 1. `execute()` with `rows_affected()` check is cleaner.

---

### 5. `get_all_homies` / `get_all_restaurants`

These are straightforward and well-written:
```sql
select id, user_id, name from homies where user_id = ?
```

No issues beyond the standard trait/params visibility warnings.

---

## Index Review

### Existing Indexes

| Index | Table | Columns | Assessment |
|-------|-------|---------|------------|
| `homies_user_uindex` | `homies` | `(user_id, id)` | ❌ Redundant — `id` is already the PK. `(user_id, name)` already exists. Drop this. |
| `homies_name_uindex` | `homies` | `(user_id, name)` | ✅ Good |
| `restaurant_user_uindex` | `restaurants` | `(user_id, id)` | ❌ Redundant — same reason as above. Drop this. |
| `restaurant_name_uindex` | `restaurants` | `(user_id, name)` | ✅ Good |

### Missing Indexes

| Table | Columns | Reason |
|-------|---------|--------|
| `recent_restaurants` | `(user_id, homie_id, date)` | The `get_candidates` query filters by `user_id` and `homie_id` and orders by `date`. An index would speed up the CTE significantly. |
| `homies_favorite_restaurants` | `(user_id, homie_id)` | Used in `get_candidates` join and `get_homies_favorite_restaurants`. |

---

## Data Access Patterns

### Repository Trait Pattern

Each DB operation defines a trait:

```rust
trait GetAllHomies {
    async fn get_all_homies(&self, params: UserId) -> Result<Vec<Homie>, sqlx::Error>;
}

impl GetAllHomies for Pool<Sqlite> { ... }
```

**Assessment**:
- **Pro**: Clean separation, testable in theory.
- **Con**: Massive boilerplate. Every operation needs: public fn, params struct, error enum, trait, impl. For a CLI with ~10 DB operations, this generates hundreds of lines of ceremony.
- **Con**: No mock implementations. The traits exist only for `Pool<Sqlite>`.
- **Con**: The `async_fn_in_trait` allowance is needed because the project uses Edition 2021. In Rust 1.75+, this is stable, but returning `impl Trait` in traits still has limitations.

### Alternative Patterns to Consider

1. **Plain functions with `&Pool<Sqlite>`**: Remove traits entirely for a small CLI.
2. **Generic `Repository` struct**: One struct holding `Pool<Sqlite>` with methods.
3. **`mockall` with traits**: If keeping traits, add `#[automock]` and write unit tests.

---

## Connection Pool

```rust
SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
```

For a single-user CLI, 5 connections is reasonable. SQLite handles concurrency via WAL mode (evidenced by `lunch.db-wal` and `lunch.db-shm` files in the repo).

**Note**: SQLite with WAL mode supports one writer and multiple readers. With sqlx's pool, writers may briefly contend. For a CLI, this is negligible.

---

## Migration Management

- **Framework**: `sqlx::migrate!("./migrations")`
- **State**: Stored in `_sqlx_migrations` table automatically.
- **Files**:
  - `20230603012307_inital-create.up.sql` / `.down.sql` — Main schema
  - `f/20240628044251_recipes.up.sql.sql` / `.down.sql` — Recipes (typo in filename)

**Issue**: The `f/` subdirectory is non-standard. sqlx's migrate macro recursively searches, but organizing by letter is unusual and may confuse tooling.
