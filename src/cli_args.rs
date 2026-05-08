use clap::Args;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Command
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Specify path to config file
    #[arg(
        long,
        short,
        require_equals = false,
        value_name = "config_file",
        help = "defaults to ~/.config/local/lunch.json",
        value_parser
    )]
    pub config_file: Option<String>,

    /// Specify emitting additional debug information
    #[clap(short, long, value_parser)]
    pub debug: bool,
}
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Operations related to Homies
    #[command(subcommand)]
    Homies(Homies),

    /// Operations related to Recipes
    #[command(subcommand)]
    Recipes(Recipes),

    /// Operations related to Restaurants
    #[command(subcommand)]
    Restaurants(Restaurants),

    /// Manage pantry inventory
    #[command(subcommand)]
    Pantry(Pantry),

    /// Meal planning
    #[command(subcommand)]
    Plan(Plan),

    /// Meal plan templates
    #[command(subcommand)]
    Template(Template),

    #[clap(
        name = "pick-lunch",
        visible_alias = "p",
        about = "Pick what to eat for lunch"
    )]
    Pick,
}

#[derive(Args, Debug)]
pub struct AddHomiesArgs {
    /// Name of homie
    #[clap(name = "homie's name", value_parser)]
    pub homies_name: String,
}

#[derive(Subcommand, Debug)]
pub enum Homies {
    /// Add a homie
    Add(AddHomiesArgs),

    /// Delete a homie
    Delete {
        /// name of homie to delete
        #[clap(short, value_parser)]
        homies_name: String,
    },

    /// Rename a homie
    Rename {
        /// name of homie
        #[clap(short, value_parser)]
        homies_name: String,
        /// new name
        #[clap(short, value_parser)]
        updated_name: String,
    },

    #[command(subcommand)]
    Restaurants(AddRestaurant),

    #[command(subcommand)]
    RecentRestaurant(AddRestaurant),

    /// Manage Favorites for a Homie Interactively
    #[clap(visible_alias = "i")]
    Interactive,
}

#[derive(Subcommand, Debug)]
pub enum Recipes {
    /// Add a recipe
    #[clap(visible_alias = "a")]
    Add {
        /// Name of recipe
        #[clap(name = "recipe name", value_parser)]
        recipe_name: String,
        /// Description
        #[clap(long, value_parser)]
        description: Option<String>,
        /// Prep time in minutes
        #[clap(long, value_parser)]
        prep_time: Option<i32>,
        /// Cook time in minutes
        #[clap(long, value_parser)]
        cook_time: Option<i32>,
        /// Number of servings
        #[clap(long, value_parser)]
        servings: Option<i32>,
    },

    /// Delete a recipe
    #[clap(visible_alias = "d")]
    Delete {
        /// name of recipe to delete
        #[clap(short, value_parser)]
        recipe_name: String,
    },

    /// View a recipe
    #[clap(visible_alias = "v")]
    View {
        /// name of recipe
        #[clap(short, value_parser)]
        recipe_name: String,
    },

    /// List all recipes
    #[clap(visible_alias = "l")]
    List,

    /// Import a recipe from URL
    #[clap(visible_alias = "i")]
    Import {
        /// URL to import from
        #[clap(name = "url", value_parser)]
        url: String,
    },

    /// Add a step to a recipe
    AddStep {
        /// Name of recipe
        #[clap(name = "recipe name", value_parser)]
        recipe_name: String,
        /// Step number
        #[clap(name = "step number", value_parser)]
        step_number: i32,
        /// Instruction text
        #[clap(name = "instruction", value_parser)]
        instruction: String,
    },

    /// Add an ingredient to a recipe
    AddIngredient {
        /// Name of recipe
        #[clap(name = "recipe name", value_parser)]
        recipe_name: String,
        /// Name of ingredient
        #[clap(name = "ingredient name", value_parser)]
        ingredient_name: String,
        /// Quantity
        #[clap(long, value_parser)]
        qty: f64,
        /// Measure (cup, tbsp, tsp, oz, lb, g, kg, ml, l, each, qty, count)
        #[clap(long, value_parser)]
        measure: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum Pantry {
    /// Add an ingredient to pantry
    #[clap(visible_alias = "a")]
    Add {
        /// Name of ingredient
        #[clap(name = "ingredient name", value_parser)]
        ingredient_name: String,
        /// Quantity
        #[clap(long, value_parser)]
        qty: f64,
        /// Measure
        #[clap(long, value_parser)]
        measure: String,
    },

    /// Update pantry quantity
    #[clap(visible_alias = "u")]
    Update {
        /// Name of ingredient
        #[clap(name = "ingredient name", value_parser)]
        ingredient_name: String,
        /// Quantity
        #[clap(long, value_parser)]
        qty: f64,
        /// Measure
        #[clap(long, value_parser)]
        measure: String,
    },

    /// Remove an ingredient from pantry
    #[clap(visible_alias = "r")]
    Remove {
        /// Name of ingredient
        #[clap(name = "ingredient name", value_parser)]
        ingredient_name: String,
        /// Measure (optional)
        #[clap(long, value_parser)]
        measure: Option<String>,
    },

    /// List pantry ingredients
    #[clap(visible_alias = "l")]
    List,
}

#[derive(Subcommand, Debug)]
pub enum Plan {
    /// Add a recipe to the meal plan
    #[clap(visible_alias = "a")]
    Add {
        /// Recipe name
        #[clap(long, value_parser)]
        recipe: String,
        /// Date (YYYY-MM-DD)
        #[clap(long, value_parser)]
        date: String,
        /// Meal slot (breakfast, lunch, dinner, snack)
        #[clap(long, value_parser, default_value = "dinner")]
        slot: String,
    },

    /// Remove a meal plan entry
    #[clap(visible_alias = "r")]
    Remove {
        /// Entry ID
        #[clap(long, value_parser)]
        id: i32,
    },

    /// View meal plan
    #[clap(visible_alias = "v")]
    View {
        /// Start date (YYYY-MM-DD)
        #[clap(long, value_parser)]
        start: Option<String>,
        /// Number of days
        #[clap(long, value_parser, default_value_t = 7)]
        days: i64,
    },

    /// Generate shopping list
    #[clap(visible_alias = "s")]
    Shop {
        /// Start date (YYYY-MM-DD)
        #[clap(long, value_parser)]
        start: Option<String>,
        /// Number of days
        #[clap(long, value_parser, default_value_t = 7)]
        days: i64,
    },
}

#[derive(Subcommand, Debug)]
pub enum Template {
    /// Add a meal plan template
    #[clap(visible_alias = "a")]
    Add {
        /// Template name
        #[clap(long, value_parser)]
        name: String,
        /// Day of week (0=Sunday through 6=Saturday)
        #[clap(long, value_parser)]
        day: i32,
        /// Meal slot
        #[clap(long, value_parser, default_value = "dinner")]
        slot: String,
        /// Recipe name
        #[clap(long, value_parser)]
        recipe: String,
    },

    /// Remove a template
    #[clap(visible_alias = "r")]
    Remove {
        /// Template ID
        #[clap(long, value_parser)]
        id: i32,
    },

    /// List templates
    #[clap(visible_alias = "l")]
    List,

    /// Apply templates to meal plan
    #[clap(visible_alias = "apply")]
    Apply {
        /// Start date (YYYY-MM-DD)
        #[clap(long, value_parser)]
        start: Option<String>,
        /// Number of days
        #[clap(long, value_parser, default_value_t = 7)]
        days: i64,
    },
}

#[derive(Subcommand, Debug)]
pub enum AddRestaurant {
    /// Add a restaurant
    #[clap(visible_alias = "a")]
    Add {
        /// Homie Name
        #[clap(name = "homie name", value_parser)]
        homie_name: String,
        /// Name of restaurant
        #[clap(name = "restaurant name", value_parser)]
        restaurant_name: String,
    },

    /// Delete a restaurant
    #[clap(visible_alias = "d")]
    Delete {
        /// Homie Name
        #[clap(name = "homie name", value_parser)]
        homie_name: String,
        /// name of restaurant to delete
        #[clap(name = "restaurant name", value_parser)]
        restaurant_name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum Restaurants {
    /// Add a restaurant
    #[clap(visible_alias = "a")]
    Add {
        /// Name of restaurant
        #[clap(name = "restaurant name", value_parser)]
        restaurant_name: String,
    },

    /// Delete a restaurant
    #[clap(visible_alias = "d")]
    Delete {
        /// name of restaurant to delete
        #[clap(short, value_parser)]
        restaurant_name: String,
    },

    /// Rename a restaurant
    #[clap(visible_alias = "r")]
    Rename {
        /// name of restaurant
        #[clap(short, value_parser)]
        restaurant_name: String,
        /// new name
        #[clap(short, value_parser)]
        updated_name: String,
    },
}
