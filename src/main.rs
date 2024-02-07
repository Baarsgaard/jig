mod commands;
mod config;
mod hooks;
mod interactivity;
mod repo;

use clap::{command, CommandFactory, Parser, Subcommand};
use color_eyre::eyre::{Result, WrapErr};
use color_eyre::owo_colors::OwoColorize;
use commands::{shared::ExecCommand, *};
use config::Config;
use hooks::{is_git_hook, Hook};
use inquire::InquireError;

#[derive(Parser)]
#[command(author, version, about = "A Jira CLI integration with Git", long_about = None)]
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
    /// Generate completion script
    #[command(arg_required_else_help(true))]
    Completion(Completion),
    /// List config file locations
    Configs(PrintConfigs),
    /// Install git commit-msg hook
    Hook(Hooks),
    /// Initialise config file(s)
    Init(InitConfig),
    /// Create a work log entry on a Jira issue
    #[command(aliases = ["w", "wl", "l", "log"], arg_required_else_help(true))]
    Worklog(Worklog),
    /// Move ticket through transitions
    #[command(aliases = ["t"])]
    Transition(Transition),
    /// Open issue in your browser
    #[command(alias = "o")]
    Open(Open),
    /// Interactively send JQL queries to Jira when tab is pressed
    #[command(alias = "q")]
    #[cfg(debug_assertions)]
    Query(Query),
    /// Download and install latest version
    #[command(aliases = ["u", "update"])]
    Upgrade(Upgrade),
}

impl Commands {
    async fn exec(cfg: Result<Config>) -> Result<String> {
        let args = Cli::parse();

        match args.command {
            Commands::Assign(assign) => assign.exec(&cfg?).await,
            Commands::Branch(branch) => branch.exec(&cfg?).await,
            Commands::Comment(comment) => comment.exec(&cfg?).await,
            Commands::Completion(completion) => completion.exec(&mut Cli::command()),
            Commands::Configs(print_config) => print_config.exec(&cfg?).await,
            Commands::Hook(hooks) => hooks.install(),
            Commands::Init(init) => init.init(),
            Commands::Worklog(worklog) => worklog.exec(&cfg?).await,
            Commands::Transition(transition) => transition.exec(&cfg?).await,
            Commands::Open(open) => open.exec(&cfg?).await,
            #[cfg(debug_assertions)]
            Commands::Query(query) => query.exec(&cfg?).await,
            Commands::Upgrade(upgrade) => upgrade.exec(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cfg = config::Config::load().wrap_err("Failed to load config");

    if let Some(githook) = is_git_hook()? {
        match githook.exec(&cfg?).await {
            Ok(_) => (),
            Err(e) => {
                match e.root_cause().downcast_ref::<InquireError>() {
                    Some(InquireError::OperationInterrupted)
                    | Some(InquireError::OperationCanceled) => std::process::exit(1),
                    _ => {
                        // Add better comments from old hook
                        eprintln!("{}", format!("Error:\n   {}", e).bright_red());
                        std::process::exit(1);
                    }
                }
            }
        }
    } else {
        let res = Commands::exec(cfg).await;
        match res {
            Ok(msg) => println!("{}", msg),
            Err(e) => match e.root_cause().downcast_ref::<InquireError>() {
                Some(InquireError::OperationInterrupted)
                | Some(InquireError::OperationCanceled) => std::process::exit(1),
                _ => Err(e)?,
            },
        }
    };

    // Fix windows prompt overriding last line of output
    #[cfg(target_os = "windows")]
    println!("");

    Ok(())
}
