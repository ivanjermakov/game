use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
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

    let results = WineLauncher::match_name(&cli.game_name)?;
    WineLauncher::run(results.first().context("no matches")?)?;

    Ok(())
}

trait Matcher {
    fn match_name(name: &str) -> Result<Vec<PathBuf>>;
}

trait Executor {
    fn run(path: &Path) -> Result<()>;
}

struct WineLauncher;

impl Matcher for WineLauncher {
    fn match_name(name: &str) -> Result<Vec<PathBuf>> {
        let args = [
            "-i",
            "-t",
            "f",
            "-e",
            "exe",
            "-g",
            &format!("*{name}*"),
            "/D/Program Files/",
        ];
        let mut command = Command::new("fd");
        command.args(args);
        println!("{:?}", command);
        let output = command.output()?;
        let result = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(PathBuf::from)
            .collect();
        Ok(result)
    }
}

impl Executor for WineLauncher {
    fn run(path: &Path) -> Result<()> {
        println!("running {path:?}");
        let mut command = Command::new("wine");
        if let Some(parent) = path.parent() {
            command.current_dir(parent);
        }
        let exec_name = path.file_name().context("no file name")?;
        command.args([exec_name]);
        println!("spawning {command:?}");
        command.spawn()?;
        Ok(())
    }
}
