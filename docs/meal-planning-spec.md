# Meal Planning Feature Specification

## Overview

Extend `lunch_picker` from a restaurant randomizer into a full meal planning system. Users will maintain a **pantry** (ingredient inventory), a **recipe book** with structured ingredients and instructions, and generate **weekly meal plans** that produce **shopping lists**. Recipes can be imported from URLs via web scraping to reduce manual data entry.

This feature builds on the existing (partial) recipes schema and integrates with the current `homies`, `restaurants`, and `favorites` model.

---

## Goals

1. **Inventory Management**: Track what ingredients the user has on hand with quantities and units.
2. **Recipe Management**: CRUD for recipes with structured ingredients, quantities, measures, instructions, and metadata (prep time, servings, source URL).
3. **Recipe Import**: Parse recipe data from common cooking websites (schema.org/Recipe JSON-LD) via URL.
4. **Meal Planning**: Assign recipes to calendar days and meal slots (breakfast, lunch, dinner).
5. **Shopping List Generation**: Diff planned recipe ingredients against pantry inventory to produce a shopping list.
6. **Integration**: Meal plans can feed into the existing "pick" flow as an alternative to restaurants.

## Non-Goals

1. **Nutritional Analysis**: No calorie/macro tracking (out of scope).
2. **Multi-user Real-time Sync**: Still a single-user CLI; no server or sharing.
3. **Image Storage**: No image blobs or file attachments.
4. **Complex Unit Conversion**: No automatic conversion between weight/volume (e.g., cups to grams). Users pick from a fixed enum.
5. **Mobile/Web UI**: Still CLI-only.
6. **Authentication/Authorization**: Continues to use the hardcoded `CLI_USER_ID = 1` pattern.

---

## User Stories

### Pantry / Inventory
- As a user, I want to add ingredients to my pantry so I know what I have on hand.
- As a user, I want to update pantry quantities when I buy groceries or use ingredients.
- As a user, I want to remove ingredients from my pantry when they are depleted.
- As a user, I want to view my entire pantry so I can plan meals around what I already have.

### Recipes
- As a user, I want to add a recipe manually with a name, ingredient list, and instructions.
- As a user, I want to import a recipe from a URL so I don't have to type it in.
- As a user, I want to view a recipe's details including ingredients and steps.
- As a user, I want to edit a recipe if I change ingredients or instructions.
- As a user, I want to delete a recipe I no longer use.
- As a user, I want to mark certain recipes as favorites so they appear more often in picks.
- As a user, I want to tag recipes (e.g., "vegetarian", "quick", "italian") for filtering.

### Meal Planning
- As a user, I want to plan meals for the upcoming week by assigning recipes to days.
- As a user, I want to generate a shopping list from my meal plan that shows only what I need to buy.
- As a user, I want the shopping list to subtract ingredients I already have in my pantry.
- As a user, I want to run `pick` and have the option to pick from my meal plan instead of restaurants.

---

## Functional Requirements

### FR1: Ingredient Inventory (Pantry)

**FR1.1 - Add Ingredient to Pantry**
Given a user provides an ingredient name, quantity, and measure
When the ingredient does not already exist in the global `ingredients` table
Then it is created in `ingredients` and added to the user's `pantry_ingredients`.

**FR1.2 - Update Pantry Quantity**
Given an ingredient already exists in the user's pantry
When the user provides a new quantity
Then the quantity is updated (overwrite, not append).

**FR1.3 - Remove from Pantry**
Given an ingredient exists in the user's pantry
When the user requests removal
Then the row is deleted from `pantry_ingredients`.

**FR1.4 - View Pantry**
Given the user requests their pantry
Then return all ingredients sorted alphabetically with quantity and measure.

**FR1.5 - Ingredient Deduplication**
Given two pantry entries with the same ingredient and same measure
They should be stored as a single row with combined quantity.
Given two pantry entries with the same ingredient but different measures
They should remain separate rows (no automatic conversion).

### FR2: Recipe Management

**FR2.1 - Create Recipe (Manual)**
Given a recipe name, optional description, optional steps (ordered list of instructions), optional prep_time, optional servings, and a list of ingredients (name + qty + measure)
When the recipe name is unique for this user
Then create the recipe, create any missing global ingredients, link them via `recipe_ingredients`, and store steps in `recipe_steps`.

**FR2.2 - View Recipe**
Given a recipe name or ID
When requested
Then return the full recipe: name, description, steps (ordered by step_number), prep_time, servings, source_url, and ingredient list.

**FR2.3 - Update Recipe**
Given a recipe ID and updated fields
When the user saves changes
Then update the recipe header and replace the ingredient list atomically.

**FR2.4 - Delete Recipe**
Given a recipe ID
When deleted
Then cascade delete `recipe_ingredients`, `meal_plan_entries`, and `homies_favorite_recipes` referencing it.

**FR2.5 - List Recipes**
Given no filters
Return all recipes for the user sorted by name.
Given a tag filter
Return only recipes matching the tag(s).
Given a "favorites only" flag
Return only recipes marked as favorites for the selected homie(s).

**FR2.6 - Tag Recipes**
Given a recipe and a tag name
When the tag is added
Then store it in `recipe_tags`.

### FR3: Recipe Import from URL

**FR3.1 - Schema.org/Recipe Detection**
Given a URL
When the page contains a `<script type="application/ld+json">` block with `@type: "Recipe"`
Then parse `name`, `recipeIngredient`, `recipeInstructions`, `prepTime`, `cookTime`, `totalTime`, `recipeYield`, `description`, and `url`.

**FR3.2 - Ingredient Parsing**
Given a `recipeIngredient` string like "2 cups flour" or "3 tbsp olive oil"
Then attempt to parse: quantity (numeric), measure (from known enum), ingredient name (remainder).
If parsing fails, store the raw string in a `raw_ingredient` field and flag for manual cleanup.

**FR3.3 - Duplicate Prevention**
Given a URL that has already been imported for this user
When import is attempted
Then reject with "Recipe already imported" and return the existing recipe ID.

**FR3.4 - Fallback on Failure**
Given a URL with no schema.org Recipe data
When import is attempted
Then return a clear error: "No recipe data found at URL. Supported sites: ..."

**FR3.5 - Supported Sites (Initial)**
- Allnrecipes
- Food Network
- Bon Appetit
- Serious Eats
- Any site using schema.org/Recipe JSON-LD (generic)

### FR4: Meal Planning

**FR4.1 - Assign Recipe to Day**
Given a recipe ID, a date, and an optional meal slot (default: "dinner")
When assigned
Then create or update a `meal_plan_entries` row for that user + date + slot.

**FR4.2 - View Weekly Plan**
Given a start date (default: today)
When requested
Then return all `meal_plan_entries` for the 7-day window with recipe details.

**FR4.3 - Unassign Recipe**
Given a meal plan entry ID
When deleted
Then remove the assignment.

**FR4.4 - Generate Shopping List**
Given a date range (default: next 7 days)
When requested
Then:
1. Aggregate all `recipe_ingredients` for planned recipes in the range.
2. Group by ingredient + measure.
3. Sum quantities per measure.
4. Subtract `pantry_ingredients` quantities for matching ingredient + measure.
5. Return only rows where needed qty > pantry qty (or pantry qty is 0).
6. Sort by ingredient name.

**FR4.5 - Mark Shopping List Items as Purchased**
Given a shopping list item (ingredient + measure + qty)
When marked purchased
Then upsert the quantity into `pantry_ingredients` and optionally clear the planned entry flag.

**FR4.6 - Apply Templates to Meal Plan**
Given a date range and existing `meal_plan_templates`
When `plan apply-templates` is run
Then for each day in the range, if the day's weekday matches a template's `day_of_week`, create a `meal_plan_entries` row with the template's `recipe_id` and `meal_slot`, unless that slot is already occupied.

### FR5: Integration with Existing Pick Flow

**FR5.1 - Pick from Meal Plan**
Given the user runs `pick` with a `--source=recipes` flag (or interactive choice)
When candidates are generated
Then use the upcoming week's `meal_plan_entries` as the candidate pool (instead of `get_candidate_restaurants`).

**FR5.2 - Homie Recipe Favorites**
Given the existing `homies_favorite_recipes` table
When picking from recipes
Then respect the same intersection logic: only recipes that are favorites of ALL home homies.

**FR5.3 - Recent Recipe Exclusion**
Given the existing `recent_recipes` table
When picking from recipes
Then exclude the 5 most recent recipe picks per homie within 21 days.

---

## Data Model Changes

### New Tables

```sql
-- Ingredients already exist in recipes migration, but pantry_ingredients needs user_id FK fix
-- (Current schema has duplicate FK to users in shopping_cart)

CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL CHECK (length(name) > 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);

CREATE TABLE recipe_tags (
    recipe_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (recipe_id, tag_id),
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes(id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE meal_plan_entries (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    date DATE NOT NULL,
    meal_slot TEXT NOT NULL DEFAULT 'dinner' CHECK (meal_slot IN ('breakfast', 'lunch', 'dinner', 'snack')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes(id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, date, meal_slot)
);

-- Add missing columns to recipes
-- (These should be added via a new migration since recipes table already exists)
-- ALTER TABLE recipes ADD COLUMN description TEXT;
-- ALTER TABLE recipes ADD COLUMN instructions TEXT;
-- ALTER TABLE recipes ADD COLUMN prep_time INTEGER; -- minutes
-- ALTER TABLE recipes ADD COLUMN cook_time INTEGER; -- minutes
-- ALTER TABLE recipes ADD COLUMN servings INTEGER;
-- ALTER TABLE recipes ADD COLUMN source_url TEXT;
-- ALTER TABLE recipes ADD COLUMN imported_at TIMESTAMP;
```

### New Tables (from Decisions)

```sql
CREATE TABLE recipe_steps (
    id INTEGER PRIMARY KEY,
    recipe_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    step_number INTEGER NOT NULL CHECK (step_number > 0),
    instruction TEXT NOT NULL CHECK (length(instruction) > 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes(id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(recipe_id, step_number)
);

CREATE TABLE meal_plan_templates (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL CHECK (length(name) > 0),
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 0 AND 6), -- 0=Sunday
    meal_slot TEXT NOT NULL DEFAULT 'dinner' CHECK (meal_slot IN ('breakfast', 'lunch', 'dinner', 'snack')),
    recipe_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes(id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, day_of_week, meal_slot)
);
```

### Modified Tables

| Table | Change | Reason |
|-------|--------|--------|
| `recipes` | Add `description`, `prep_time`, `cook_time`, `servings`, `source_url`, `imported_at` | Richer recipe data. **Note:** `instructions` moved to `recipe_steps` table per decision. |
| `recipe_ingredients` | Add `raw_ingredient TEXT` nullable | Store unparsed ingredient strings from imports |
| `shopping_cart` | Remove duplicate `user_id` FK | Current schema has `FOREIGN KEY (user_id)` twice |
| `pantry_ingredients` | Ensure `user_id` FK is correct | Verify against current schema |

### Index Recommendations

```sql
CREATE INDEX meal_plan_date_index ON meal_plan_entries(user_id, date);
CREATE INDEX meal_plan_template_user_index ON meal_plan_templates(user_id, day_of_week);
CREATE INDEX recipe_tags_recipe_index ON recipe_tags(recipe_id);
CREATE INDEX recipe_steps_recipe_index ON recipe_steps(recipe_id);
CREATE INDEX recipes_source_url_index ON recipes(user_id, source_url); -- For duplicate import detection
```

---

## CLI Interface Design

```
lunch_picker [COMMAND]

# Pantry / Ingredients
pantry add <ingredient_name> --qty=<n> --measure=<measure>
pantry update <ingredient_name> --qty=<n> --measure=<measure>
pantry remove <ingredient_name> [--measure=<measure>]
pantry list

# Recipes
recipes add <recipe_name> [--description="..."] [--instructions="..."]
recipes import <url>
recipes view <recipe_name>
recipes edit <recipe_name> [--name="..."] [--description="..."]
recipes delete <recipe_name>
recipes list [--tag=<tag>] [--favorite-for=<homie_name>]
recipes add-ingredient <recipe_name> <ingredient_name> --qty=<n> --measure=<measure>
recipes remove-ingredient <recipe_name> <ingredient_name>
recipes tag <recipe_name> <tag_name>
recipes untag <recipe_name> <tag_name>

# Meal Planning
plan add --recipe=<name> --date=<YYYY-MM-DD> [--slot=dinner]
plan remove --date=<YYYY-MM-DD> [--slot=dinner]
plan view [--start=<YYYY-MM-DD>] [--days=7]
plan shop [--start=<YYYY-MM-DD>] [--days=7]
plan buy <ingredient_name> --qty=<n> --measure=<measure>  # mark as purchased -> add to pantry
plan apply-templates [--start=<YYYY-MM-DD>] [--days=7]  # auto-fill plan from templates

# Meal Plan Templates
template add --name="Taco Night" --day=monday --slot=dinner --recipe="Tacos"
template remove --day=monday --slot=dinner
template list

# Pick (existing)
pick [--source=restaurants|recipes]  # default stays restaurants for backward compat
```

### Interactive Mode Extensions

When running `lunch_picker` without arguments (the default `AppState::work` flow):
1. After selecting "Who's home?", prompt:
   - "What are we planning for?" → [Restaurants, Recipes (Meal Plan)]
2. If "Recipes" is selected:
   - Show upcoming meal plan entries as candidates.
   - Fall back to recipe favorites if no plan exists.
   - Use `recent_recipes` exclusion logic.
3. After selection, record as `recent_recipes` for all home homies.

---

## Recipe Import: URL Parsing Specification

### Request
```
Command: recipes import <url>
```

### Processing Flow
1. **Fetch**: HTTP GET the URL with a timeout of 15s and a browser-like User-Agent.
2. **Extract JSON-LD**: Find all `<script type="application/ld+json">` blocks.
3. **Parse JSON**: For each block, parse as JSON. Handle `@graph` arrays.
4. **Find Recipe**: Locate object where `@type == "Recipe"` (or `@type` array contains `"Recipe"`).
5. **Map Fields**:
   - `name` → `recipes.name`
   - `description` → `recipes.description`
   - `recipeIngredient` (array of strings) → parse each into `recipe_ingredients`
   - `recipeInstructions` → parse into structured steps and store in `recipe_steps`.
     - If `recipeInstructions` is an array of objects with `@type: "HowToStep"`, use `text` field for each step.
     - If it is a single string or array of strings, split into one step per string.
   - `prepTime` / `cookTime` / `totalTime` (ISO 8601 duration, e.g., `PT30M`) → parse to minutes
   - `recipeYield` (e.g., "4 servings") → extract integer
   - `url` or canonical URL → `recipes.source_url`
6. **Ingredient Parsing** (per `recipeIngredient` string):
   - Use regex: `^((?:\d+\s+)?(?:\d+\/\d+|\d*\.?\d+))?\s*(cup|tbsp|tsp|oz|lb|g|kg|ml|l|each|qty|count)?\s*(.+)$`
   - If match: extract quantity (as float), measure (map to enum), name (trimmed).
   - If no match: store full string in `recipe_ingredients.raw_ingredient`, set quantity=1, measure='each'.
7. **Save**: Insert recipe + ingredients + recipe_ingredients in a transaction.
8. **Deduplication**: Before insert, check `recipes.source_url` for this user. If exists, abort.

### Error Handling
| Scenario | Error Message |
|----------|--------------|
| HTTP timeout/failure | "Failed to fetch URL: {details}" |
| No JSON-LD found | "No recipe data found. The site may not support schema.org/Recipe markup." |
| JSON-LD found but no Recipe type | "No recipe data found at this URL." |
| Already imported | "Recipe already imported: {recipe_name}" |
| DB error | "Failed to save recipe: {db_error}" |

---

## Shopping List Generation Algorithm

```sql
-- Aggregate needed ingredients from meal plan
WITH planned_recipes AS (
    SELECT recipe_id
    FROM meal_plan_entries
    WHERE user_id = ?
      AND date BETWEEN ? AND ?
),
needed AS (
    SELECT
        ri.ingredient_id,
        ri.measure,
        SUM(ri.quantity) AS total_needed
    FROM recipe_ingredients ri
    JOIN planned_recipes pr ON ri.recipe_id = pr.recipe_id
    GROUP BY ri.ingredient_id, ri.measure
),
pantry AS (
    SELECT ingredient_id, measure, quantity AS total_have
    FROM pantry_ingredients
    WHERE user_id = ?
)
SELECT
    i.name,
    n.measure,
    MAX(0, n.total_needed - COALESCE(p.total_have, 0)) AS qty_to_buy
FROM needed n
JOIN ingredients i ON n.ingredient_id = i.id
LEFT JOIN pantry p ON n.ingredient_id = p.ingredient_id AND n.measure = p.measure
WHERE n.total_needed > COALESCE(p.total_have, 0)
ORDER BY i.name;
```

---

## Technical Constraints

1. **No new runtime dependencies** for core features (SQLx, clap, dialoguer, etc. are sufficient).
2. **Recipe import requires an HTTP client** — add `reqwest` with `rustls-tls` and `json` features.
3. **HTML/JSON-LD parsing** — add `scraper` (HTML parsing) and `serde_json` (already present).
4. **SQLite remains the sole database**.
5. **All new queries must compile with `sqlx` offline mode** — run `cargo sqlx prepare` after migration.
6. **Keep trait-based repository pattern** for consistency with existing codebase, even if it is boilerplate-heavy.
7. **Hardcoded `CLI_USER_ID = 1`** pattern continues; no auth changes.

---

## Decisions (Resolved)

| Question | Decision |
|----------|----------|
| Recipe instructions format | **Structured steps table** — `recipe_steps` with `step_number` and `instruction` |
| Shopping list persistence | **Computed on-demand** — stateless, no persistent shopping_cart usage |
| Recurring meal plan templates | **Included in v1** — `meal_plan_templates` table with day-of-week + meal_slot |
| Unit conversion | **No automatic conversion** — pantry and recipe must use same measure to match |
| Unsupported recipe sites | **Schema.org/Recipe JSON-LD only** — no plain HTML scraping |

## Open Questions

1. **Unit conversion for shopping list?**
   - If pantry has "flour: 500g" and recipe needs "flour: 2 cups", they are different measures.
   - *Decision: No automatic conversion. Show both lines or require user to normalize units.*

2. **How should recipe import handle sites without schema.org?**
   - *Decision: v1 supports schema.org/Recipe JSON-LD only. Plain HTML scraping is too brittle.*

---

## Acceptance Criteria

- [ ] User can add, update, remove, and list pantry ingredients via CLI.
- [ ] User can create a recipe manually with ingredients and instructions.
- [ ] User can import a recipe from a supported URL and see parsed ingredients.
- [ ] Duplicate URL import is rejected with a clear message.
- [ ] User can assign recipes to specific days and view a weekly meal plan.
- [ ] User can create recurring meal plan templates (e.g., "Monday dinner = Tacos").
- [ ] `plan apply-templates` auto-fills the meal plan from templates without overwriting existing entries.
- [ ] `plan shop` generates a shopping list that correctly subtracts pantry quantities.
- [ ] `pick --source=recipes` (or interactive choice) uses meal plan + favorites + recent exclusion.
- [ ] All new DB operations follow the existing trait-based repository pattern.
- [ ] New migrations run successfully and are compatible with existing data.
- [ ] `cargo sqlx prepare` succeeds (offline query validation).
- [ ] `cargo test` passes (including new integration tests for pantry, recipes, meal plan).
- [ ] Interactive mode (`lunch_picker` with no args) offers "Recipes" as a pick source.

---

## Implementation Phases

### Phase 1: Foundation
- Fix existing recipes migration (SQLite compatibility, `name` type issue — current migration uses `name` type which is PostgreSQL-specific).
- Add missing columns to `recipes`.
- Create `tags`, `recipe_tags`, `meal_plan_entries` tables.
- Implement `pantry_ingredients` CRUD.

### Phase 2: Recipe Management
- Implement manual recipe CRUD (create, view, list, delete).
- Implement recipe ingredient management (add/remove ingredients to a recipe).
- Implement recipe tagging.

### Phase 3: Recipe Import
- Add `reqwest` and `scraper` dependencies.
- Implement URL fetch + JSON-LD extraction.
- Implement ingredient string parsing.
- Implement import command with deduplication.

### Phase 4: Meal Planning
- Implement `meal_plan_entries` CRUD (add, remove, view).
- Implement `meal_plan_templates` CRUD (add, remove, list).
- Implement `plan apply-templates` command.
- Implement shopping list generation query.
- Implement `plan shop` and `plan buy` commands.

### Phase 5: Integration
- Wire recipe picking into `AppState::work` interactive flow.
- Implement `recent_recipes` exclusion for recipe picks.
- Implement `homies_favorite_recipes` intersection for recipe picks.
- Add CLI arg `--source=recipes` to `pick`.

### Phase 6: Polish
- Add integration tests for all new features.
- Update README with new commands.
- Run `cargo sqlx prepare`.
