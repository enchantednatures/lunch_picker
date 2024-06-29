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
