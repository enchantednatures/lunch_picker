# Implementation Plan: Meal Planning Feature

## Overview

This plan implements the meal planning specification across 6 phases. The work builds on the existing (partial) recipes schema and follows the codebase's trait-based repository pattern, newtype wrappers, and CLI structure. Key decisions: structured recipe steps, stateless shopping lists, and recurring meal plan templates.

## Architecture Decisions

**Pattern:** Continue existing Feature Module + Trait-Based Repository pattern
**Rationale:** The codebase is disciplined and consistent. All new code must match the existing `features/<domain>/<action>.rs` structure with `Params` structs, traits, and `Pool<Sqlite>` implementations.

**Database:** SQLite via sqlx, with offline query validation via `.sqlx/` metadata
**CLI:** Extend `clap` derive-based subcommands; extend `dialoguer` interactive flows
**HTTP:** `reqwest` with `rustls-tls` for recipe URL import
**HTML Parsing:** `scraper` for JSON-LD extraction from recipe pages

## Component Breakdown

### Component 1: Database Schema & Migrations
**Purpose:** Create and alter tables to support pantry, structured recipe steps, tags, meal plans, and templates.
**Inputs:** Existing schema (`migrations/20230603012307_inital-create.up.sql`, `migrations/f/20240628044251_recipes.up.sql`)
**Outputs:** New migration file, updated `.sqlx/` query metadata
**Dependencies:** None (foundation)
**Complexity:** Medium — must be SQLite-compatible and fix existing PostgreSQL-isms in the recipes migration
**Risks:** The existing recipes migration uses `name` type and `serial` (PostgreSQL). Must create a SQLite-compatible migration that adds missing columns/tables without breaking existing data.
**Files to create/modify:**
- `migrations/20250507000000_meal_planning.up.sql`
- `migrations/20250507000000_meal_planning.down.sql`

### Component 2: Domain Models & Validation
**Purpose:** Newtype wrappers, validation, and domain types for ingredients, recipes, steps, tags, meal plans, and templates.
**Inputs:** Existing patterns in `features/restaurants/models.rs`, `features/homies/models.rs`
**Outputs:** Type-safe domain models with `TryFrom` validation
**Dependencies:** None (foundation)
**Complexity:** Low-Medium — repetitive but must be consistent
**Risks:** None significant
**Files to create/modify:**
- `src/features/ingredients/models.rs`
- `src/features/recipes/models.rs` (extend existing)
- `src/features/recipe_steps/models.rs`
- `src/features/tags/models.rs`
- `src/features/meal_plan_entries/models.rs`
- `src/features/meal_plan_templates/models.rs`
- `src/features/pantry_ingredients/models.rs`

### Component 3: Pantry CRUD
**Purpose:** Add, update, remove, and list pantry ingredients.
**Inputs:** `pantry_ingredients` table, `ingredients` global table
**Outputs:** Public functions + traits + DB implementations
**Dependencies:** Component 2 (models)
**Complexity:** Low
**Risks:** Ingredient deduplication logic (same ingredient + measure = combine qty)
**Files to create/modify:**
- `src/features/pantry_ingredients/add.rs`
- `src/features/pantry_ingredients/update.rs`
- `src/features/pantry_ingredients/remove.rs`
- `src/features/pantry_ingredients/list.rs`
- `src/features/pantry_ingredients.rs` (module file)

### Component 4: Recipe Management CRUD
**Purpose:** Create, view, list, update, delete recipes with structured steps and ingredients.
**Inputs:** `recipes`, `recipe_ingredients`, `recipe_steps`, `ingredients` tables
**Outputs:** Public functions + traits + DB implementations
**Dependencies:** Component 2 (models), Component 3 (ingredients/pantry patterns)
**Complexity:** Medium — recipe creation is a multi-table transaction (recipe + steps + ingredients)
**Risks:** Transaction wrapping for atomic recipe + steps + ingredients insert
**Files to create/modify:**
- `src/features/recipes/create.rs` (extend existing stub)
- `src/features/recipes/get.rs`
- `src/features/recipes/list.rs`
- `src/features/recipes/update.rs`
- `src/features/recipes/delete.rs`
- `src/features/recipe_steps/create.rs`
- `src/features/recipe_steps/list.rs`
- `src/features/recipe_steps/delete.rs`
- `src/features/recipe_ingredients/add.rs`
- `src/features/recipe_ingredients/remove.rs`

### Component 5: Recipe Tags
**Purpose:** Tag recipes and filter by tags.
**Inputs:** `tags`, `recipe_tags` tables
**Outputs:** Public functions + traits + DB implementations
**Dependencies:** Component 2 (models), Component 4 (recipes)
**Complexity:** Low
**Files to create/modify:**
- `src/features/tags/create.rs`
- `src/features/tags/list.rs`
- `src/features/recipe_tags/add.rs`
- `src/features/recipe_tags/remove.rs`
- `src/features/recipe_tags/list_by_recipe.rs`

### Component 6: Recipe Import Service
**Purpose:** Fetch URL, extract schema.org/Recipe JSON-LD, parse ingredients, save recipe.
**Inputs:** URL string, `reqwest`, `scraper`, `serde_json`
**Outputs:** Imported `Recipe` or error
**Dependencies:** Component 4 (recipe creation), Component 2 (models)
**Complexity:** High — HTTP fetching, HTML parsing, regex-based ingredient parsing, ISO 8601 duration parsing
**Risks:**
- Ingredient parsing regex will be imperfect; must handle unparsed strings gracefully
- Sites may block or rate-limit; need reasonable timeout and user-agent
- JSON-LD may be malformed or use `@graph` arrays
**Files to create/modify:**
- `src/features/recipes/import.rs`
- `src/features/recipes/import/ingredient_parser.rs`
- `src/features/recipes/import/schema_org.rs`
- `Cargo.toml` (add `reqwest`, `scraper`)

### Component 7: Meal Plan Entry CRUD
**Purpose:** Assign/unassign recipes to days and view weekly plan.
**Inputs:** `meal_plan_entries` table
**Outputs:** Public functions + traits + DB implementations
**Dependencies:** Component 2 (models), Component 4 (recipes)
**Complexity:** Low
**Files to create/modify:**
- `src/features/meal_plan_entries/create.rs`
- `src/features/meal_plan_entries/delete.rs`
- `src/features/meal_plan_entries/list.rs`
- `src/features/meal_plan_entries.rs` (module file)

### Component 8: Meal Plan Templates
**Purpose:** Create recurring templates and apply them to generate plan entries.
**Inputs:** `meal_plan_templates` table
**Outputs:** Public functions + traits + DB implementations
**Dependencies:** Component 2 (models), Component 7 (meal plan entries)
**Complexity:** Medium — apply-templates must respect existing entries and map weekday integers
**Files to create/modify:**
- `src/features/meal_plan_templates/create.rs`
- `src/features/meal_plan_templates/delete.rs`
- `src/features/meal_plan_templates/list.rs`
- `src/features/meal_plan_templates/apply.rs`
- `src/features/meal_plan_templates.rs` (module file)

### Component 9: Shopping List Generator
**Purpose:** Aggregate planned recipe ingredients, subtract pantry, return what to buy.
**Inputs:** `meal_plan_entries`, `recipe_ingredients`, `pantry_ingredients`, `ingredients` tables
**Outputs:** Shopping list rows (ingredient name, measure, qty to buy)
**Dependencies:** Component 3 (pantry), Component 7 (meal plan entries)
**Complexity:** Medium — SQL CTE for aggregation and LEFT JOIN subtraction
**Risks:** SQL query must be validated by sqlx offline prepare
**Files to create/modify:**
- `src/features/shopping_list/generate.rs`
- `src/features/shopping_list/buy.rs`
- `src/features/shopping_list.rs` (module file)

### Component 10: Recipe Pick Integration
**Purpose:** Allow `pick` to use recipes/meal plans instead of restaurants, with favorites and recents.
**Inputs:** Existing `get_candidate_restaurants` logic, `recent_recipes`, `homies_favorite_recipes`
**Outputs:** `get_candidate_recipes` function and integration into `AppState::work`
**Dependencies:** Component 4 (recipes), Component 7 (meal plan entries)
**Complexity:** Medium — must replicate the weighted random + recent exclusion logic for recipes
**Risks:** The existing `get_candidates.rs` has known issues (hardcoded `user_id = 1`, `unwrap()`). Must not replicate those bugs.
**Files to create/modify:**
- `src/features/recipes/get_candidates.rs`
- `src/features/recents/add_recent_recipe.rs`
- `src/main.rs` (wire into AppState)
- `src/interaction.rs` (add interactive prompts)

### Component 11: CLI Args & Command Dispatch
**Purpose:** Extend clap CLI with all new subcommands and flags.
**Inputs:** Existing `cli_args.rs`
**Outputs:** New subcommands for pantry, recipes, plan, template, pick --source
**Dependencies:** All above components
**Complexity:** Low-Medium — repetitive but must match clap patterns
**Files to create/modify:**
- `src/cli_args.rs`
- `src/main.rs` (command dispatch match arms)

### Component 12: Integration Tests
**Purpose:** Test new features end-to-end with SQLite test DB.
**Inputs:** Existing test patterns in `tests/`
**Outputs:** New test files with fixtures
**Dependencies:** All above components
**Complexity:** Medium
**Files to create/modify:**
- `tests/pantry.rs`
- `tests/recipes.rs`
- `tests/meal_plan.rs`
- `tests/fixtures/pantry.sql`
- `tests/fixtures/recipes_full.sql`
- `tests/fixtures/meal_plan.sql`

## Execution Sequence

### Phase 1: Foundation
1. **Database Schema & Migrations** — Create new migration; fix SQLite compatibility issues in existing recipes migration.
2. **Domain Models & Validation** — Create all newtype wrappers and validation types.
3. **Update `features.rs` module declarations** — Register new feature modules.

### Phase 2: Pantry & Recipe Core
4. **Pantry CRUD** — Full ingredient inventory management.
5. **Recipe Management CRUD** — Create, view, list, delete recipes with steps and ingredients.
6. **Recipe Tags** — Tagging and filtering.

### Phase 3: Recipe Import
7. **Recipe Import Service** — Add HTTP deps, implement fetch + parse + save.

### Phase 4: Meal Planning
8. **Meal Plan Entry CRUD** — Assign recipes to days.
9. **Meal Plan Templates** — Recurring templates and apply logic.
10. **Shopping List Generator** — Aggregate, subtract pantry, output list.

### Phase 5: Integration
11. **Recipe Pick Integration** — Wire recipes into existing pick flow with favorites and recents.
12. **CLI Args & Command Dispatch** — Expose all features via CLI.

### Phase 6: Polish
13. **Integration Tests** — Write tests for all new features.
14. **SQLx Offline Prepare** — `cargo sqlx prepare` for CI builds.
15. **README Update** — Document new commands.

## Dependency Graph

```
Database Schema
    ↓
Domain Models
    ↓
    ├── Pantry CRUD
    ├── Recipe Management CRUD
    │       ↓
    │   Recipe Tags
    │       ↓
    │   Recipe Import Service
    │       ↓
    │   Meal Plan Entry CRUD
    │       ↓
    │   Meal Plan Templates
    │       ↓
    │   Shopping List Generator
    │       ↓
    │   Recipe Pick Integration
    │       ↓
    │   CLI Args & Command Dispatch
    │       ↓
    │   Integration Tests
```

## Technical Specifications

### New Dependencies (Cargo.toml)

```toml
[dependencies]
# ... existing deps ...
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
scraper = { version = "0.19", default-features = false }
```

### Error Handling Strategy

- Continue using `thiserror` for domain errors and `anyhow` for application-level propagation.
- Recipe import errors: `RecipeImportError` enum with variants for HTTP, Parse, Duplicate, DB.
- Pantry operations: `PantryError` enum with variants for Validation, NotFound, AlreadyExists.
- All DB trait implementations map `sqlx::Error` to domain-specific errors (match existing patterns in `create_restaurant.rs`).

### Testing Strategy

**Unit Tests:**
- Ingredient string parsing regex (many test cases for edge cases)
- ISO 8601 duration parsing (e.g., `PT30M` → 30 minutes)
- Model validation (empty names, invalid measures)

**Integration Tests:**
- Pantry: add → list → update → remove → list empty
- Recipes: create with steps and ingredients → view → list → delete
- Recipe import: mock HTTP response with JSON-LD → verify parsed recipe
- Meal plan: add entry → view week → apply templates → verify generated entries
- Shopping list: plan recipes → generate list → verify pantry subtraction

**Mocking:**
- For recipe import unit tests, mock `reqwest` responses or abstract behind a `HttpClient` trait.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Existing recipes migration is PostgreSQL-specific and won't run on SQLite | High | High | Create a new SQLite-compatible migration; do not rely on the broken `f/20240628044251_recipes` migration. |
| sqlx offline prepare fails on complex CTEs | Medium | High | Simplify queries if needed; test with `cargo sqlx prepare` early in Phase 1. |
| Ingredient parsing regex is too brittle | Medium | Medium | Fallback to `raw_ingredient` field; allow user editing post-import. |
| Recipe import HTTP requests blocked by sites | Medium | Low | Use browser-like User-Agent; document supported sites; provide manual fallback. |
| Feature bloat — too many new modules | Low | Medium | Keep each module small and focused; use subdirectories only where needed. |

## Critical Path

The critical path is: **Schema → Models → Recipe CRUD → Recipe Import → Meal Plan Entries → Shopping List → CLI Integration → Tests**

Any delay in schema or models blocks everything downstream. Recipe import is the highest-risk component and should be prototyped early (spike in Phase 3).

## Open Questions

1. **Should we fix the existing broken recipes migration or ignore it?**
   - The `f/20240628044251_recipes.up.sql` uses PostgreSQL types (`serial`, `name` type, `create type measure as enum`). It likely never ran on SQLite.
   - *Decision:* Create a new SQLite-compatible migration in the standard `migrations/` directory. The old one can be ignored or deleted.

2. **Should we refactor the trait-based repository pattern?**
   - The existing pattern is boilerplate-heavy with no mock implementations.
   - *Decision:* Keep the pattern for consistency. Do not refactor existing code as part of this feature.

3. **Should `plan shop` output to stdout or a file?**
   - *Decision:* Default to stdout (human-readable list). Add optional `--format=json` or `--output=file.txt` if needed later.
