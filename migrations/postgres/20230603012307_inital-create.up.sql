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
    foreign key (user_id) references users (id) on delete cascade
);

create unique index homies_user_uindex on homies (user_id, id);
create unique index homies_name_uindex on homies (user_id, name);

create table restaurants
(
    id serial primary key,
    user_id integer not null,
    name name not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    foreign key (user_id) references users (id) on delete cascade
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
    foreign key (restaurant_id, user_id) references restaurants (
        id, user_id
    ) on delete cascade,
    foreign key (homie_id, user_id) references homies (
        id, user_id
    ) on delete cascade,
    primary key (homie_id, restaurant_id, date)
);


create table homies_favorite_restaurants
(
    homie_id integer not null,
    restaurant_id integer not null,
    user_id integer not null,
    foreign key (restaurant_id, user_id) references restaurants (
        id, user_id
    ) on delete cascade,
    foreign key (homie_id, user_id) references homies (
        id, user_id
    ) on delete cascade,
    primary key (homie_id, restaurant_id)
);

create view homies_recents_restaurants_view as
select
    restaurant_id,
    homie_id,
    user_id,
    date,
    rank
from (select
    restaurant_id,
    homie_id,
    user_id,
    date,
    rank() over (partition by homie_id order by date desc) as rank
from recent_restaurants) as t
where
    rank <= 5
    and date > current_date - interval '21 days';

insert into users (id)
values (1);
