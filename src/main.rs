use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(value_parser, help = "Game name")]
    game_name: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("Hello, world!");
    println!("{}", cli.game_name);
    Ok(())
}
