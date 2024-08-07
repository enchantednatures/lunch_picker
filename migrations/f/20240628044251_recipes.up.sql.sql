create table recipes
(
    id serial primary key,
    user_id integer not null,
    name name not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    foreign key (user_id) references users (id) on delete cascade
);

create unique index recipes_user_uindex on recipes (user_id, id);
create unique index recipes_name_uindex on recipes (user_id, name);

create table recent_recipes
(
    recipe_id integer not null,
    homie_id integer not null,
    user_id integer not null,
    date date not null default current_date,
    created_at timestamp not null default current_timestamp,
    foreign key (recipe_id, user_id) references recipes (
        id, user_id
    ) on delete cascade,
    foreign key (homie_id, user_id) references homies (
        id, user_id
    ) on delete cascade,
    primary key (homie_id, recipe_id, date)
);



create table homies_favorite_recipes
(
    homie_id integer not null,
    recipe_id integer not null,
    user_id integer not null,
    foreign key (recipe_id, user_id) references recipes (id, user_id),
    foreign key (homie_id, user_id) references homies (id, user_id),
    primary key (homie_id, recipe_id)
);

create type measure as enum (
    'cup',
    'tbsp',
    'tsp',
    'oz',
    'lb',
    'g',
    'kg',
    'ml',
    'l',
    'each',
    'qty',
    'count'
);


create table ingredients
(
    id serial primary key,
    name name not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);


create table recipe_ingredients
(
    recipe_id integer not null,
    ingredient_id integer not null,
    quantity integer not null,
    measure measure not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    primary key (recipe_id, ingredient_id),
    foreign key (recipe_id) references recipes (id) on delete cascade,
    foreign key (ingredient_id) references ingredients (id) on delete cascade
);

create table shopping_cart
(
    user_id integer not null,
    ingredient_id integer not null,
    quantity integer not null,
    measure measure not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    primary key (user_id, ingredient_id),
    foreign key (user_id) references users (id) on delete cascade,
    foreign key (ingredient_id) references ingredients (id) on delete cascade,
    foreign key (user_id) references users (id) on delete cascade
);


create table pantry_ingredients
(
    user_id integer not null,
    ingredient_id integer not null,
    quantity integer not null,
    measure measure not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    primary key (user_id, ingredient_id),
    foreign key (user_id) references users (id) on delete cascade,
    foreign key (ingredient_id) references ingredients (id) on delete cascade
);


