# Performance Analysis

## Summary

The `lunch_picker` CLI is not performance-critical — it runs interactively, operates on small datasets (dozens of homies and restaurants), and executes a handful of queries per invocation. However, several patterns introduce unnecessary overhead and could cause issues as the dataset grows or if the application is ever extended (e.g., a web API).

---

## Current Performance Characteristics

### Query Count per Default Flow

| Step | Query | Approx. Time |
|------|-------|--------------|
| Get all homies | 1 SELECT | < 1ms |
| Get home homies (interactive) | N/A (in-memory) | User-dependent |
| Get candidate restaurants | 1 complex CTE SELECT | 1–5ms |
| Select restaurant (interactive) | N/A | User-dependent |
| Add recent for homies | 1 INSERT with CTE | 1–2ms |

**Total DB time**: < 10ms for typical datasets.

### Dataset Scale

With SQLite as the backend, the application will perform well up to:
- **Homies**: 1,000+ (unlikely for a lunch group)
- **Restaurants**: 1,000+ (possible for a city)
- **Recent picks**: 10,000+ (5 per homie × years of lunches)

At these scales, the current queries remain fast due to SQLite's B-tree indexes and the simplicity of the data model.

---

## Identified Performance Issues

### 1. JSON Serialization for Integer Arrays

**Location**: `get_candidates.rs`, `add_recent_restaurant.rs`

```rust
let homie_ids = serde_json::to_string(&home_homies.iter().map(|h| h.as_i32()).collect::<Vec<i32>>())
    .expect("...");
```

**Impact**: Converting a `Vec<i32>` to JSON string just to use SQLite's `json_each()` function adds:
- Allocation for the JSON string
- JSON parsing overhead in SQLite
- Extra parameter binding

**Measurement**: For 5 homies, negligible. For 100+ homies, ~0.1ms overhead.

**Better Approach**: Use a temporary table or dynamic `IN` clause. With sqlx, dynamic `IN` is tricky, but possible:

```rust
// Generate query with correct number of placeholders
let placeholders = homie_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
let query = format!(
    "SELECT value as homie_id FROM json_each('[{}]')",  // Or use a CTE with VALUES
    placeholders
);
// ... bind each id
```

Alternatively, skip CTEs and use a subquery with `IN`:
```sql
where homie_id in (?, ?, ?, ?, ?)
```

For a CLI, the JSON approach is acceptable but inelegant.

---

### 2. Complex CTE in `get_candidates`

**Location**: `get_candidates.rs`

The query has 4 CTEs, 3 nested subqueries, and a window function.

**Impact**: SQLite's query planner handles this reasonably, but:
- The view `homies_recents_restaurants_view` computes `rank()` for ALL recents, then filters. The view is not materialized, so the window function runs on every query.
- The `random()` in `ORDER BY` prevents SQLite from using indexes for sorting.

**Optimization**: Add a covering index:
```sql
create index idx_recent_restaurants_user_homie_date 
    on recent_restaurants(user_id, homie_id, date);
```

This lets SQLite efficiently find recent restaurants for specific homies without scanning the entire table.

---

### 3. `fetch_one` for Single-Row Inserts

**Location**: `restaurants.rs`, `remove_homies_favorite_restaurant.rs`

Using `fetch_one` with `RETURNING *` on INSERT/DELETE allocates memory for the returned row even when the data is discarded.

**Impact**: Minimal for a CLI, but unnecessary.

**Fix**: Use `execute()` without `RETURNING` when the row data isn't needed.

---

### 4. Connection Pool for a CLI

**Location**: `main.rs`

```rust
SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
```

**Impact**: Maintaining a connection pool for a short-lived CLI process is overkill. SQLite connection establishment is fast (~1ms). A single connection (`SqliteConnection`) would suffice and eliminate pool overhead.

**Counter-argument**: The pool enables concurrent queries if the app were ever converted to a long-running service (e.g., a web API). For now, it's premature optimization.

**Recommendation**: Keep the pool for architectural consistency, but consider `max_connections(1)` for the CLI to reduce file descriptor usage and locking contention.

---

### 5. `block_on` in Drop

**Location**: `main.rs`

```rust
impl Drop for AppState {
    fn drop(&mut self) {
        futures::executor::block_on(self.db.close());
    }
}
```

**Impact**: Forces synchronous execution of an async operation. If the runtime is busy or the close operation hangs (e.g., WAL checkpoint), this blocks the thread. In a CLI, this adds latency to process exit.

**Fix**: Explicit `db.close().await` in `main()` before returning.

---

### 6. WAL File Growth

SQLite WAL mode (`lunch.db-wal` present) keeps changes in a separate file until checkpointed. For a CLI that opens and closes the DB frequently:
- The WAL file grows until SQLite auto-checkpoints (default: 1000 pages).
- A large WAL slows down opens and increases disk usage.

**Fix**: Force a checkpoint on clean shutdown:
```rust
sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&db).await?;
```

---

### 7. Build Size Optimizations

**File**: `Cargo.toml`

The release profile is already well-tuned:
```toml
[profile.release]
panic = 'abort'
codegen-units = 1
opt-level = 's'
lto = true
```

**Additional options**:
```toml
strip = true  # Strip debug symbols
```

This can reduce binary size by several MB.

---

## Scalability Limits

### SQLite Limits

| Resource | SQLite Limit | App Reality |
|----------|-------------|-------------|
| DB size | 281 TB | Will never hit |
| Tables per DB | 2 billion | ~5 tables |
| Rows per table | 2^64 | Will never hit |
| Max SQL length | 1 GB | Queries are small |
| Max host params | 32766 | Max ~5 homies |

SQLite is more than sufficient for this application.

### Bottlenecks if Extended

If the app were extended to a multi-user web service:

1. **SQLite concurrency**: SQLite handles ~1 write transaction at a time. Multiple concurrent users would queue.
   - **Fix**: Migrate to PostgreSQL (see `depreciate-postgres` branch).

2. **No caching**: Every request hits the DB.
   - **Fix**: Cache homie/restaurant lists in memory (e.g., `moka` cache).

3. **Synchronous random selection**: The `random()` in SQL is fine, but for more complex algorithms (e.g., machine learning recommendations), computation would shift to Rust.
   - **Fix**: Pre-compute candidate sets in background jobs.

---

## Recommendations

| Priority | Action | Expected Impact |
|----------|--------|-----------------|
| Low | Add `strip = true` to release profile | -2–5 MB binary size |
| Low | Add `PRAGMA wal_checkpoint` on shutdown | Faster subsequent opens |
| Low | Change `max_connections(5)` to `max_connections(1)` | Reduced file descriptors |
| Medium | Add `idx_recent_restaurants_user_homie_date` | Faster candidate queries at scale |
| Medium | Replace `fetch_one` with `execute` where possible | Slightly lower latency |
| High | Remove `block_on` in Drop | Reliable shutdown |
