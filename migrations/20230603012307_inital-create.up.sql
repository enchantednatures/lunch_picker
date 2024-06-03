create table users
(
    id serial primary key,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);

create table recipes
(
    id serial primary key,
    user_id integer not null,
    name varchar(255) not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);

create unique index recipes_name_uindex on recipes (user_id, name);

create table restaurants
(
    id serial primary key,
    user_id integer not null,
    name varchar(255) not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);

create unique index restaurant_name_uindex on restaurants (user_id, name);

create table recent_meals
(
    id serial primary key,
    user_id integer not null,
    name varchar(255) not null,
    created_at timestamp not null default current_timestamp
);
create table homies
(
    id serial primary key,
    user_id integer not null,
    name varchar(255) not null
);

create unique index homies_name_uindex on homies (user_id, name);

create table homies_favorite_recipes
(
    homie_id integer not null,
    recipe_id integer not null,
    primary key (homie_id, recipe_id),
    foreign key (homie_id) references homies (id),
    foreign key (recipe_id) references recipes (id)
);

create table homies_favorite_restaurants
(
    homie_id integer not null,
    restaurant_id integer not null,
    primary key (homie_id, restaurant_id),
    foreign key (homie_id) references homies (id),
    foreign key (restaurant_id) references restaurants (id)
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
    name varchar(255) not null,
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
    foreign key (recipe_id) references recipes (id),
    foreign key (ingredient_id) references ingredients (id)
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
    foreign key (user_id) references users (id),
    foreign key (ingredient_id) references ingredients (id)
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
    foreign key (user_id) references users (id),
    foreign key (ingredient_id) references ingredients (id)
);
