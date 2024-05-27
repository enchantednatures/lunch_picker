-- Add up migration script here
CREATE TABLE recipes
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name       VARCHAR(255)                      NOT NULL,
    created_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE UNIQUE INDEX recipes_name_uindex ON recipes (name);

CREATE TABLE restaurants
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name       VARCHAR(255)                      NOT NULL,
    created_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX restaurant_name_uindex ON restaurants (name);

CREATE TABLE recent_meals
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name       VARCHAR(255)                      NOT NULL,
    created_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE homies
(
    id   INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name VARCHAR(255)                      NOT NULL
);

CREATE UNIQUE INDEX homies_name_uindex ON homies (name);

CREATE TABLE homies_favorite_recipes
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    homie_id  INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    FOREIGN KEY (homie_id) REFERENCES homies (id),
    FOREIGN KEY (recipe_id) REFERENCES recipes (id)
);

CREATE UNIQUE INDEX homies_favorite_recipes_homie_id_recipe_id_uindex ON homies_favorite_recipes (homie_id, recipe_id);

CREATE TABLE homies_favorite_restaurants
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    homie_id  INTEGER NOT NULL,
    restaurant_id INTEGER NOT NULL,
    FOREIGN KEY (homie_id) REFERENCES homies (id),
    FOREIGN KEY (restaurant_id) REFERENCES restaurants (id)
);
CREATE UNIQUE INDEX homies_favorite_restaurants_homie_id_restaurant_id_uindex ON homies_favorite_restaurants (homie_id, restaurant_id);
