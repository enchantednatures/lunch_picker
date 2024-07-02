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
            // default_missing_value = "always",
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
    // #[clap(visible_alias = "a")]
    Add(AddHomiesArgs),

    /// Delete a homie
    // #[clap(visible_alias = "d")]
    Delete {
        /// name of homie to delete
        #[clap(short, value_parser)]
        homies_name: String,
    },

    /// Rename a homie
    // #[clap(visible_alias = "r")]
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
    /// Add a recipie
    #[clap(visible_alias = "a")]
    Add {
        /// Name of recipie
        #[clap(name = "recipe name", value_parser)]
        recipe_name: String,
    },

    /// Delete a recipie
    #[clap(visible_alias = "d")]
    Delete {
        /// name of recipie to delete
        #[clap(short, value_parser)]
        recipe_name: String,
    },

    /// Rename a recipie
    #[clap(visible_alias = "r")]
    Rename {
        /// name of recipie
        #[clap(short, value_parser)]
        recipe_name: String,
        /// new name
        #[clap(short, value_parser)]
        updated_name: String,
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
