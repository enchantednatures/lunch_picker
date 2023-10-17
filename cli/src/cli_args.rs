use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Command
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Specify emitting additional debug information
    #[clap(short, long, value_parser)]
    pub debug: bool,
}
#[derive(Subcommand, Debug)]
pub enum Commands{
    /// Operations related to Homies
    #[command(subcommand)]
    Homies(Homies),
} 


#[derive(Subcommand, Debug)]
pub enum Homies {
    /// Add a homie
    #[clap(visible_alias = "a")]
    Add {
        /// Name of homie
        #[clap(name="homie's name", value_parser)]
        homies_name: String,
    },

    /// Delete a homie
    #[clap(visible_alias = "d")]
    Delete {
        /// name of homie to delete
        #[clap(short, value_parser)]
        homies_name: String,
    },

    /// Rename a homie
    #[clap(visible_alias = "r")]
    Rename {
        /// name of homie
        #[clap(short, value_parser)]
        homies_name: String,
        /// new name
        #[clap(short, value_parser)]
        updated_name: String,
    },
}

