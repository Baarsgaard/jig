use clap::{command, Parser, Subcommand};
use color_eyre::eyre::{Result, WrapErr};
use commands::*;
use config::Config;

mod commands;
mod config;
mod interactivity;
mod repo;

pub trait ExecCommand {
    fn exec(self, cfg: &Config) -> Result<String>;
}

#[derive(Parser)]
#[command(author, version, about = "A Jira CLI integration with Git", long_about = None)]
#[command(args_conflicts_with_subcommands = true, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Assign user to issue
    #[command(alias = "a")]
    Assign(Assign),
    /// Create and checkout branch using issue key with(out) summary as branch name
    #[command(alias = "b")]
    Branch(Branch),
    /// Create comment on a Jira Issue
    #[command(alias = "c")]
    Comment(Comment),
    /// List config file locations
    Configs(PrintConfigs),
    /// Initialise config file(s)
    Init(InitConfig),
    /// Create a work log entry on a Jira issue
    #[command(alias = "l")]
    Log(Worklog),
    /// Move ticket through transitions
    #[command(alias = "m")]
    Move(Transition),
    /// Open issue using BROWSER var
    #[command(alias = "o")]
    Open(Open),
    /// Interactively send JQL queries to Jira when tab is pressed
    #[command(alias = "q")]
    #[cfg(debug_assertions)]
    Query(Query),
    /// Download and install latest version
    #[command(alias = "u")]
    #[cfg(debug_assertions)]
    Upgrade(Upgrade),
}

impl Commands {
    fn exec(args: Cli) -> Result<String> {
        let cfg = config::Config::load().wrap_err("Failed to load config");

        match args.command {
            Commands::Assign(assign) => assign.exec(&cfg?),
            Commands::Branch(branch) => branch.exec(&cfg?),
            Commands::Comment(comment) => comment.exec(&cfg?),
            Commands::Configs(print_config) => print_config.exec(&cfg?),
            Commands::Init(init) => init.init(),
            Commands::Log(worklog) => worklog.exec(&cfg?),
            Commands::Move(transition) => transition.exec(&cfg?),
            Commands::Open(open) => open.exec(&cfg?),
            #[cfg(debug_assertions)]
            Commands::Query(query) => query.exec(&cfg?),
            #[cfg(debug_assertions)]
            Commands::Upgrade(upgrade) => upgrade.exec(&cfg?),
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();

    match Commands::exec(args) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
