# Holistic Code Review

## Executive Summary

The `lunch_picker` codebase demonstrates solid Rust fundamentals and good separation of concerns, but suffers from significant boilerplate repetition, inconsistent error handling, dead code accumulation, and several runtime risks (panics, unwraps, hardcoded values). The trait-based repository pattern is well-intentioned but currently provides no value since no alternative implementations exist. The codebase appears to be in a transitional state between a prototype and a polished application.

---

## Strengths

### 1. Domain Modeling with Newtypes
The use of `HomieId`, `RestaurantId`, `UserId`, `HomiesName`, and `RestaurantName` prevents common bugs where raw `i32` or `String` values get mixed up. Validation on `TryFrom<String>` ensures names are non-empty and trimmed.

### 2. Feature-Based Organization
Code is cleanly grouped by domain concept (`homies`, `restaurants`, `recents`, `homies_favorites`), making it easy to locate related functionality.

### 3. SQLx Compile-Time Query Checking
The `.sqlx/` metadata enables offline query verification. This catches SQL errors at compile time rather than runtime.

### 4. Tracing Instrumentation
DB queries and user interactions are instrumented with `tracing`, providing structured observability. Spans include contextual data like home homie counts.

### 5. CI/CD Coverage
GitHub Actions cover build, test, formatting, clippy, and automated releases across multiple platforms.

### 6. Clean CLI Design
`clap` derive macros produce a well-structured CLI with subcommands, aliases, and help text.

---

## Critical Issues

### 1. Panics in User-Facing Code
**File**: `src/interaction.rs` (lines 184, 253)

```rust
pub async fn get_favorite_restaurants(homies: &[Homie]) -> Result<Vec<&Homie>> {
    if homies.is_empty() {
        tracing::error!("No homies found");
        panic!(); // ❌ Never panic in user-facing interactive code
    }
    ...
}
```

**Impact**: Application crashes if the user somehow reaches this state.
**Fix**: Return a proper `Err(...)` with a descriptive message.

---

### 2. Unwrap in Core Business Logic
**File**: `src/features/restaurants/get_candidates.rs` (line 93)

```rust
.fetch_all(self)
.await
.unwrap(); // ❌ Will panic on any DB error
```

**Impact**: Any DB error (locked DB, disk full, malformed query) crashes the app.
**Fix**: Propagate the error properly.

---

### 3. Hardcoded User ID
**File**: `src/main.rs` (line 45)

```rust
const CLI_USER_ID: i32 = 1;
```

This constant is used throughout the application. The database schema supports multiple users, but the application is hardcoded to user 1 everywhere.

**Impact**: No multi-user support despite schema readiness.
**Fix**: Accept user_id from config, env var, or CLI flag.

Also present in SQL:
**File**: `src/features/restaurants/get_candidates.rs` (line 64)
```sql
where r.user_id = 1  -- ❌ Hardcoded, ignores the parameter bound below
```

---

### 4. Risky Drop Implementation
**File**: `src/main.rs` (lines 116–121)

```rust
impl Drop for AppState {
    fn drop(&mut self) {
        futures::executor::block_on(self.db.close());
        opentelemetry::global::shutdown_tracer_provider();
    }
}
```

**Impact**: `block_on` inside `Drop` is dangerous. If called from an async context, it can panic or deadlock. The `futures` crate wildcard dependency is also problematic.
**Fix**: Use `tokio::runtime::Handle::try_current()` or, better, avoid `Drop` and explicitly clean up before `main` exits.

---

### 5. No Input Validation for Restaurants
**File**: `src/features/restaurants/create_restaurant.rs`

Unlike `create_homie` which requires `TryInto<HomiesName>`, `create_restaurant` accepts a raw `String` with no validation. Empty or whitespace-only restaurant names can be inserted (though the DB CHECK constraint will catch them, resulting in a poor error experience).

**Fix**: Apply `RestaurantName` validation consistently, matching the homie pattern.

---

## Major Issues

### 6. Inconsistent Error Handling

There are **three** different error handling styles in the codebase:

| Style | Location | Example |
|-------|----------|---------|
| `thiserror` derive | Most features | `CreateHomieError`, `CreateRestaurantError` |
| Manual `Display` | `features/recipes/create.rs` | `CreateRecipeError` |
| `anyhow` | `main.rs`, `interaction.rs` | `Result<()>` |

**Impact**: Cognitive overhead, inconsistent error messages, recipes module cannot integrate cleanly.
**Fix**: Standardize on `thiserror` for domain errors and `anyhow` for application boundaries.

---

### 7. Private Interface Warnings
**Files**: Multiple feature modules

Many traits expose private param structs in public trait methods:

```rust
pub trait CreateHomie {
    async fn create_homie(&self, params: CreateHomieParams<'_>) -> ...;
    //                                          ^^^^^^^^^^^^^^^^^ private!
}
```

This generates compiler warnings (`private_interfaces`) and makes the API confusing.
**Fix**: Make params structs `pub(crate)` or inline them into the trait methods.

---

### 8. Dead and Incomplete Code

| File / Module | Issue |
|---------------|-------|
| `src/features/recipes/` | Entire module dead — commented out in `features.rs`, empty `main.rs` handler |
| `src/features/get_homie_by_name.rs` | Incomplete — trait defined but no impl, `.unwrap()` on `try_into` |
| `src/features.rs` lines 7–10 | Empty modules: `read_homie {}`, `update_homie {}`, `delete_homie {}`, `remove_favorite_from_homie {}` |
| `src/main.rs` lines 47–50 | Commented-out `HomiePaging` trait |
| `src/main.rs` lines 274–275 | Commented-out cleanup code |

---

### 9. Fragile Name-Based Lookups

Several SQL queries join by `name` instead of ID:

**File**: `src/features/homies_favorites/restaurants.rs`
```sql
join restaurants r on r.name = ? and r.user_id = ?
where h.name = ? and h.user_id = ?
```

**Impact**: Names are not immutable identifiers. If a restaurant is renamed, all related queries break conceptually. Also, names are not guaranteed unique per user (only `(user_id, name)` is unique), so `limit 1` is used as a band-aid.
**Fix**: Accept IDs in internal APIs; use names only at the CLI boundary for resolution.

---

### 10. Delete Using `fetch_one` Instead of `execute`

**File**: `src/features/homies_favorites/remove_homies_favorite_restaurant.rs` (line 135)

```rust
sqlx::query!(r#"delete ... returning *"#)
    .fetch_one(self)  // ❌ Should be .execute() for DELETE
```

**Impact**: Semantically confusing. `fetch_one` implies expecting a row, but the operation is a deletion. While `RETURNING` makes it valid, it's unnecessary complexity.
**Fix**: Use `execute()` without `RETURNING *` unless the deleted row data is needed.

---

### 11. Wildcard Dependency
**File**: `Cargo.toml` (line 34)

```toml
futures = { version = "*", ... }
```

**Impact**: Unpredictable builds, potential breaking changes on `cargo update`.
**Fix**: Pin to a specific version (e.g., `0.3`).

---

### 12. Unused Lifetimes in Traits
**Files**: `remove_homies_favorite_restaurant.rs`, `restaurants.rs`, `add_recent_restaurant.rs`

Clippy warns about unused lifetimes in trait method signatures:
```rust
async fn remove_homies_favorite_restaurant<'a>(...)  // 'a not used
```

---

### 13. Needless Returns
**File**: `src/interaction.rs` (lines 128, 188, 209, 258, 279)

Multiple closures use explicit `return` where expression syntax suffices:
```rust
.map(|h| { return h.name.as_str(); })  // should be .map(|h| h.name.as_str())
```

---

### 14. Lifetime Syntax Warnings
**Files**: `src/features/homies/models.rs`, `src/features/restaurants/models.rs`

```rust
pub fn as_view(&self) -> HomieView {  // should be HomieView<'_>
```

---

### 15. Config Path Inconsistency
The application writes config to `~/.config/local/lunch.json` but defaults the DB to `~/.local/state/lunch.db`. The XDG Base Directory spec suggests:
- Config → `~/.config/lunch_picker/config.json`
- Data/State → `~/.local/share/lunch_picker/` or `~/.local/state/lunch_picker/`

The current paths mix scopes and don't use the application name as a directory.

---

## Minor Issues

### 16. Migration Filename Typo
**File**: `migrations/f/20240628044251_recipes.up.sql.sql`

Double `.sql` extension.

### 17. Missing Recipe Handler
**File**: `src/main.rs` (line 264)

```rust
Recipes::Add { recipe_name } => {
    // _ = create_recipe(recipe_name, 1, &app_state.db).await?;
}
```
The variable is unused and the handler is empty.

### 18. No Transaction Boundaries
Multi-row operations (like `add_recent_restaurant_for_homies`) use `execute()` without an explicit transaction. If the app crashes mid-operation, partial data may be committed.

### 19. Limited Test Coverage
Tests exist for basic CRUD but miss:
- Candidate selection logic (only one trivial test)
- Interactive flows (impossible with current design)
- Error recovery paths
- CLI argument parsing

### 20. `async_fn_in_trait` Allowance
**File**: `src/lib.rs` (line 1)

```rust
#![allow(async_fn_in_trait)]
```

This suppresses warnings but the trait methods don't use RPITIT features. The allowance is unnecessary noise.

---

## Clippy Warning Summary

Running `cargo clippy` produces 20+ warnings:
- 5× `private_interfaces`
- 4× `dead_code` (get_homie_by_name)
- 3× `extra_unused_lifetimes`
- 1× `needless_borrows_for_generic_args`
- 5× `needless_return`
- 2× `mismatched_lifetime_syntaxes`
- 1× `unused_variables` (recipe_name)

Most are auto-fixable with `cargo clippy --fix`.
