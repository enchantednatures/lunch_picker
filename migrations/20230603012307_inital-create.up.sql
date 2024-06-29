create domain name name default null constraint name_not_empty check (
    length(value) > 0 and length(trim(value)) = length(value)
);
create table users
(
    id integer primary key,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp
);

create table homies
(
    id serial primary key,
    user_id integer not null,
    name name not null,
    foreign key (user_id) references users (id)
);

create unique index homies_user_uindex on homies (user_id, id);
create unique index homies_name_uindex on homies (user_id, name);

create table recipes
(
    id serial primary key,
    user_id integer not null,
    name name not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    foreign key (user_id) references users (id)
);

create unique index recipes_user_uindex on recipes (user_id, id);
create unique index recipes_name_uindex on recipes (user_id, name);

create table restaurants
(
    id serial primary key,
    user_id integer not null,
    name name not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    foreign key (user_id) references users (id)
);

create unique index restaurant_user_uindex on restaurants (user_id, id);
create unique index restaurant_name_uindex on restaurants (user_id, name);

create table recent_restaurants
(
    restaurant_id integer not null,
    homie_id integer not null,
    user_id integer not null,
    date date not null default current_date,
    created_at timestamp not null default current_timestamp,
    foreign key (restaurant_id, user_id) references restaurants (id, user_id),
    foreign key (homie_id, user_id) references homies (id, user_id),
    primary key (homie_id, restaurant_id, date)
);


create table recent_recipes
(
    recipe_id integer not null,
    homie_id integer not null,
    user_id integer not null,
    date date not null default current_date,
    created_at timestamp not null default current_timestamp,
    foreign key (recipe_id, user_id) references recipes (id, user_id),
    foreign key (homie_id, user_id) references homies (id, user_id),
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

create table homies_favorite_restaurants
(
    homie_id integer not null,
    restaurant_id integer not null,
    user_id integer not null,
    foreign key (restaurant_id, user_id) references restaurants (id, user_id),
    foreign key (homie_id, user_id) references homies (id, user_id),
    primary key (homie_id, restaurant_id)
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
    foreign key (ingredient_id) references ingredients (id),
    foreign key (user_id) references users (id)
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
insert into users (id)
values (1);
