use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use regex::Regex;

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

    if let RunStatus::Ok = WineLauncher::run(&cli.game_name)? {
        return Ok(());
    }
    if let RunStatus::Ok = SteamLauncher::run(&cli.game_name)? {
        return Ok(());
    }

    Err(anyhow!("no launchers found"))
}

enum RunStatus {
    Ok,
    NotFound,
}

trait Executor {
    fn run(name: &str) -> Result<RunStatus>;
}

struct WineLauncher;

impl Executor for WineLauncher {
    fn run(name: &str) -> Result<RunStatus> {
        let mut command = Command::new("fd");
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
        command.args(args);
        println!("{:?}", command);
        let output = command.output()?;
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(PathBuf::from)
            .next();
        println!("running {path:?}");
        match path {
            Some(path) => {
                let mut command = Command::new("wine");
                if let Some(parent) = path.parent() {
                    command.current_dir(parent);
                }
                let exec_name = path.file_name().context("no file name")?;
                command.args([exec_name]);
                println!("spawning {command:?}");
                command.spawn()?;
                Ok(RunStatus::Ok)
            }
            None => Ok(RunStatus::NotFound),
        }
    }
}

struct SteamLauncher;

impl Executor for SteamLauncher {
    fn run(name: &str) -> Result<RunStatus> {
        #[derive(Debug)]
        struct GameInfo {
            app_id: String,
            name: String,
        }

        fn parse_manifest(path: &Path) -> Result<GameInfo> {
            let content = read_to_string(path)?;
            let app_id_line = content
                .lines()
                .find(|l| l.contains("\"appid\""))
                .context("no app id")?;
            let name_line = content
                .lines()
                .find(|l| l.contains("\"name\""))
                .context("no name")?;
            let app_id = app_id_line
                .split("\t")
                .last()
                .context("no app id")?
                .replace("\"", "");
            let name = name_line
                .split("\t")
                .last()
                .context("no name id")?
                .replace("\"", "");
            Ok(GameInfo { app_id, name })
        }

        let mut command = Command::new("find");
        let args = ["/D/SteamLibrary/", "-name", "appmanifest_*"];
        command.args(args);
        println!("{:?}", command);
        let output = command.output()?;
        let manifest_files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let game = manifest_files
            .iter()
            .map(|p| parse_manifest(p))
            .filter_map(Result::ok)
            .find(|g| {
                let re = Regex::new(&format!("(?i){}", regex::escape(name))).unwrap();
                re.is_match(&g.name)
            });
        match game {
            Some(game) => {
                let mut command = Command::new("steam");
                command.args([format!("steam://rungameid/{}", game.app_id)]);
                command.spawn()?;
                Ok(RunStatus::Ok)
            }
            None => Ok(RunStatus::NotFound),
        }
    }
}
