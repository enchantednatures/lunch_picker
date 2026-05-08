CREATE TABLE IF NOT EXISTS recipes
(
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL CHECK (
        length(name) = length(trim(name)) and length(name) > 0
    ),
    description TEXT,
    prep_time INTEGER,
    cook_time INTEGER,
    servings INTEGER,
    source_url TEXT,
    imported_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    UNIQUE (user_id, name)
);

CREATE UNIQUE INDEX IF NOT EXISTS recipes_user_uindex ON recipes (user_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS recipes_name_uindex ON recipes (user_id, name);
CREATE INDEX IF NOT EXISTS recipes_source_url_index ON recipes (user_id, source_url);

CREATE TABLE IF NOT EXISTS ingredients
(
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL CHECK (
        length(name) = length(trim(name)) and length(name) > 0
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS recipe_ingredients
(
    recipe_id INTEGER NOT NULL,
    ingredient_id INTEGER NOT NULL,
    quantity REAL NOT NULL CHECK (quantity > 0),
    measure TEXT NOT NULL CHECK (
        measure IN ('cup', 'tbsp', 'tsp', 'oz', 'lb', 'g', 'kg', 'ml', 'l', 'each', 'qty', 'count')
    ),
    raw_ingredient TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (recipe_id, ingredient_id),
    FOREIGN KEY (recipe_id) REFERENCES recipes (id) ON DELETE CASCADE,
    FOREIGN KEY (ingredient_id) REFERENCES ingredients (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS recipe_steps
(
    id INTEGER PRIMARY KEY,
    recipe_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    step_number INTEGER NOT NULL CHECK (step_number > 0),
    instruction TEXT NOT NULL CHECK (length(instruction) > 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes (id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    UNIQUE (recipe_id, step_number)
);

CREATE INDEX IF NOT EXISTS recipe_steps_recipe_index ON recipe_steps (recipe_id);

CREATE TABLE IF NOT EXISTS recent_recipes
(
    recipe_id INTEGER NOT NULL,
    homie_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes (id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (homie_id, user_id) REFERENCES homies (id, user_id) ON DELETE CASCADE,
    PRIMARY KEY (homie_id, recipe_id, date)
);

CREATE TABLE IF NOT EXISTS homies_favorite_recipes
(
    homie_id INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes (id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (homie_id, user_id) REFERENCES homies (id, user_id) ON DELETE CASCADE,
    PRIMARY KEY (homie_id, recipe_id)
);

CREATE TABLE IF NOT EXISTS pantry_ingredients
(
    user_id INTEGER NOT NULL,
    ingredient_id INTEGER NOT NULL,
    quantity REAL NOT NULL CHECK (quantity >= 0),
    measure TEXT NOT NULL CHECK (
        measure IN ('cup', 'tbsp', 'tsp', 'oz', 'lb', 'g', 'kg', 'ml', 'l', 'each', 'qty', 'count')
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, ingredient_id, measure),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (ingredient_id) REFERENCES ingredients (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags
(
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL CHECK (
        length(name) = length(trim(name)) and length(name) > 0
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS recipe_tags
(
    recipe_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (recipe_id, tag_id),
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes (id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS recipe_tags_recipe_index ON recipe_tags (recipe_id);

CREATE TABLE IF NOT EXISTS meal_plan_entries
(
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    date DATE NOT NULL,
    meal_slot TEXT NOT NULL DEFAULT 'dinner' CHECK (
        meal_slot IN ('breakfast', 'lunch', 'dinner', 'snack')
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes (id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    UNIQUE (user_id, date, meal_slot)
);

CREATE INDEX IF NOT EXISTS meal_plan_date_index ON meal_plan_entries (user_id, date);

CREATE TABLE IF NOT EXISTS meal_plan_templates
(
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL CHECK (
        length(name) = length(trim(name)) and length(name) > 0
    ),
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    meal_slot TEXT NOT NULL DEFAULT 'dinner' CHECK (
        meal_slot IN ('breakfast', 'lunch', 'dinner', 'snack')
    ),
    recipe_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recipe_id, user_id) REFERENCES recipes (id, user_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    UNIQUE (user_id, day_of_week, meal_slot)
);

CREATE INDEX IF NOT EXISTS meal_plan_template_user_index ON meal_plan_templates (user_id, day_of_week);

CREATE TABLE IF NOT EXISTS shopping_cart
(
    user_id INTEGER NOT NULL,
    ingredient_id INTEGER NOT NULL,
    quantity REAL NOT NULL CHECK (quantity > 0),
    measure TEXT NOT NULL CHECK (
        measure IN ('cup', 'tbsp', 'tsp', 'oz', 'lb', 'g', 'kg', 'ml', 'l', 'each', 'qty', 'count')
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, ingredient_id, measure),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (ingredient_id) REFERENCES ingredients (id) ON DELETE CASCADE
);
