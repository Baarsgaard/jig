use anyhow::{anyhow, Context, Result};
use clap::{command, Parser, Subcommand};

mod config;
mod interactivity;
mod jira;
mod repo;
use jira::types::{CommentRequestBody, IssueKey, WorklogAddRequestBody, WorklogDuration};
use repo::Repository;

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
    Branch {
        /// Only use ISSUE_KEY as branch name
        /// Inverts 'always_short_branch_names' setting
        #[arg(short, long)]
        short_name: bool,

        /// Skip confirmation, ignores 'always_skip_branch_confirm'
        #[arg(short, long)]
        yes: bool,

        /// Skip querying Jira for Issue summary
        #[arg(value_name = "ISSUE_KEY")]
        issue_key_input: Option<String>,
    },
    /// Initialise config file(s)
    Init {
        #[arg(short, long)]
        all: bool,
    },
    /// Create comment on a Jira Issue
    Comment {
        #[arg(value_name = "COMMENT")]
        comment_input: String,

        #[arg(value_name = "ISSUE_KEY")]
        issue_key_input: Option<String>,
    },
    /// Create a work log entry on a Jira issue
    Log {
        /// Inverts 'always_confirm_date' setting
        #[arg(short, long)]
        date: bool,

        #[arg(value_name = "DURATION")]
        duration: String,

        #[arg(value_name = "COMMENT")]
        comment_input: Option<String>,

        #[arg(value_name = "ISSUE_KEY")]
        issue_key_input: Option<String>,
    },
    /// List config file locations
    Configs,
}

impl Commands {
    fn exec(args: Cli) -> Result<String> {
        let cfg = config::Config::load().context("Failed to load config")?;

        match args.command {
            Commands::Branch {
                short_name,
                yes,
                issue_key_input,
            } => {
                let repo = Repository::open().context("Failed to open repo")?;
                let branch_name = if let Some(maybe_issue_key) = issue_key_input {
                    let issue_key = IssueKey::try_from(maybe_issue_key)?;

                    if !short_name {
                        let client = jira::client::JiraAPIClient::new(&cfg)?;
                        interactivity::query_issue_details(&client, issue_key)?.to_string()
                    } else {
                        issue_key.to_string()
                    }
                } else {
                    let client = jira::client::JiraAPIClient::new(&cfg)?;
                    let issues = interactivity::query_issues_with_retry(&client, &cfg)?;
                    let issue = interactivity::prompt_user_with_issue_select(issues)?;

                    if short_name {
                        issue.key.to_string()
                    } else {
                        let branch_name = issue.to_string().replace(' ', "_");
                        match branch_name.len() {
                            n if n > 50 => branch_name.split_at(51).0.to_owned(),
                            _ => branch_name,
                        }
                    }
                };

                let prompt_branch_confirm = cfg.always_skip_branch_confirm.unwrap_or(false);

                if !yes || prompt_branch_confirm {
                    let accept = inquire::Confirm::new(
                        format!("Create and switch to: {}", branch_name).as_str(),
                    )
                    .with_help_message("Defaults to yes")
                    .with_default(true)
                    .prompt()
                    .context("Branch confirmation aborted")?;

                    if !accept {
                        return Ok(format!("Skipping branch checkout: {}", branch_name));
                    }
                }

                repo.create_branch(branch_name.clone());
                Ok(format!("Checked out: {}", branch_name))
            }

            // TODO Implement init flow
            Commands::Init { all } => {
                let _repo = Repository::open().context("Failed to open repo")?;
                println!("Init all? {}", all);
                Ok("".to_string())
            }

            Commands::Comment {
                comment_input: comment,
                issue_key_input,
            } => {
                let repo = Repository::open().context("Failed to open repo")?;
                let client = jira::client::JiraAPIClient::new(&cfg)?;

                let issue_key = if issue_key_input.is_some() {
                    IssueKey::try_from(issue_key_input.unwrap())?
                } else {
                    interactivity::issue_from_branch_or_prompt(&client, &cfg, &repo)?.key
                };

                let comment_body = CommentRequestBody { body: comment };

                let response = client.post_comment(issue_key, comment_body)?;
                if response.status().is_success() {
                    Ok("Comment posted!".to_string())
                } else {
                    Err(anyhow!(
                        "Posting comment failed!\n{:?}",
                        response.error_for_status()
                    ))
                }
            }

            Commands::Log {
                duration,
                date,
                comment_input,
                issue_key_input,
            } => {
                let repo = Repository::open().context("Failed to open repo")?;
                let client = jira::client::JiraAPIClient::new(&cfg)?;

                let issue_key = if issue_key_input.is_some() {
                    IssueKey::try_from(issue_key_input.unwrap())?
                } else {
                    interactivity::issue_from_branch_or_prompt(&client, &cfg, &repo)?.key
                };

                let comment = if let Some(cli_comment) = comment_input {
                    cli_comment
                } else if cfg.enable_comment_prompts.unwrap_or(false) {
                    inquire::Text::new("Worklog comment:")
                        .prompt()
                        .context("Worklog comment prompt cancelled")?
                } else {
                    String::default()
                };

                let wl = WorklogAddRequestBody {
                    comment,
                    started: interactivity::get_date(&cfg, date)
                        .context("Cannot create worklog request body: field=started")?,
                    time_spent: WorklogDuration::try_from(duration)
                        .context("Cannot create worklog request body: field=time_spent")?
                        .to_string(),
                };

                match client.post_worklog(issue_key, wl.clone()) {
                    Ok(r) if r.status().is_success() => Ok("Worklog created!".to_string()),
                    Ok(r) => Err(anyhow!(
                        "Worklog creation failed!\n{:?}",
                        r.error_for_status()
                    )),
                    Err(e) => Err(anyhow!("Failed to create worklog:\n{:?}\n{:?}", wl, e)),
                }
            }
            Commands::Configs => {
                println!("At least one config file is required");
                println!(
                    "Global: {:?} exists: {}",
                    config::config_file(),
                    config::config_file().exists()
                );
                println!(
                    "workspace: {:?} exists: {}",
                    config::workspace_config_file(),
                    config::workspace_config_file().exists()
                );
                Ok("".to_string())
            }
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
