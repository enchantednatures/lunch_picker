create table ingredient
(
    id          serial                                 not null
        constraint ingredient_pk primary key,
    name        varchar(255)                           not null
);


alter sequence ingredient_id_seq restart with 1000;


create table recipe
(
    id          serial                                 not null
        constraint recipe_pk primary key,
    name        varchar(255)                           not null
);


alter sequence recipe_id_seq restart with 1000;


create table pantry
(
    id          serial                                 not null
        constraint pantry_pk primary key,

    user_id     UUID                           not null,
    name        varchar(255)                           not null
);


alter sequence pantry_id_seq restart with 1000;
