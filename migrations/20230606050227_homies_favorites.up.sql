-- Add up migration script here
CREATE TABLE homies_favorites
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    homie_id  INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    FOREIGN KEY (homie_id) REFERENCES homies (id),
    FOREIGN KEY (recipe_id) REFERENCES recipes (id)
)