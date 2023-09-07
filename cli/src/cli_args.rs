use clap::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {

 /// Name of the person to greet
    #[arg(short, long)]
    name: String,
}
