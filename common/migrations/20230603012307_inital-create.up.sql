-- Add up migration script here
CREATE TABLE recipes
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name       VARCHAR(255)                      NOT NULL,
    created_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP                         NOT NULL DEFAULT CURRENT_TIMESTAMP
);

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
CREATE TABLE homies_favorites
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    homie_id  INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    FOREIGN KEY (homie_id) REFERENCES homies (id),
    FOREIGN KEY (recipe_id) REFERENCES recipes (id)
)
