use anyhow::{Context, Result};
use clap::{command, Parser, Subcommand};
use commands::{
    branch::Branch, comment::Comment, create::Create, init_config::InitConfig, open::Open,
    print_configs::PrintConfigs, query::Query, transition::Transition, worklog::Worklog,
};
use config::Config;

mod commands;
mod config;
mod interactivity;
mod jira;
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
    /// Create and checkout branch using issue key with(out) summary as branch name
    Branch(Branch),
    /// Create comment on a Jira Issue
    Comment(Comment),
    /// List config file locations
    Configs(PrintConfigs),
    /// Create new ticket
    #[cfg(debug_assertions)]
    Create(Create),
    /// Initialise config file(s)
    #[cfg(debug_assertions)]
    Init(InitConfig),
    /// Create a work log entry on a Jira issue
    Log(Worklog),
    /// Move ticket through transitions
    Move(Transition),
    /// Open issue using BROWSER var
    Open(Open),
    /// Interactively send JQL queries to Jira
    #[cfg(debug_assertions)]
    Query(Query),
}

impl Commands {
    fn exec(args: Cli) -> Result<String> {
        let cfg = config::Config::load().context("Failed to load config")?;

        match args.command {
            Commands::Branch(branch) => branch.exec(&cfg),
            Commands::Comment(comment) => comment.exec(&cfg),
            Commands::Configs(print_config) => print_config.exec(&cfg),
            #[cfg(debug_assertions)]
            Commands::Create(create) => create.exec(&cfg),
            #[cfg(debug_assertions)]
            Commands::Init(init) => init.exec(&cfg),
            Commands::Log(worklog) => worklog.exec(&cfg),
            Commands::Move(transition) => transition.exec(&cfg),
            Commands::Open(open) => open.exec(&cfg),
            #[cfg(debug_assertions)]
            Commands::Query(query) => query.exec(&cfg),
        }
    }
}

fn main() {
    let args = Cli::parse();

    match Commands::exec(args) {
        Ok(msg) => println!("{}", msg),
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
