# lunch_picker

A rust rewrite of the poor man's lunch decider implementation

## How is this going to work?

On first run you'll give it a database type and url
it'll connect to the db and migrate the tables.

You'll then be able to add homies, restaurants, and food items.
You can then configure the homies to have a list of restaurants and food items they like.

Then you'll be able to add homies, restaurants and food items to the db.

Running shitty_lunch_picker --homies={homie1,homie2,homie3} --restaurants={restaurant1,restaurant2,restaurant3}
--food={food1,food2,food3} will add them to the db.

Then you'll be able to run shitty_lunch_picker --pick --homies={homie1,homie2,homie3}
--restaurants={restaurant1,restaurant2,restaurant3} --food={food1,food2,food3} and it'll pick a random restaurant and
food item for each homie.

It will store the last picked restaurant and food item for each homie. The last 5 picks for each homie will be excluded
from each result