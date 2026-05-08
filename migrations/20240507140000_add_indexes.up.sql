create index idx_recent_restaurants_user_homie_date
    on recent_restaurants(user_id, homie_id, date);

create index idx_homies_fav_restaurants_user_homie
    on homies_favorite_restaurants(user_id, homie_id);
