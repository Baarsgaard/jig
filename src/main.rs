use clap::{command, Parser, Subcommand};
use std::process::exit;

mod config;
mod interactivity;
mod jira;
mod repo;
use jira::types::{IssueKey, WorklogAddRequestBody, WorklogDuration};
use repo::Repository;

#[derive(Parser)]
#[command(author, version, about = "A Jira CLI integration with Git", long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Gix does not support switching branch/worktree
    Branch {
        /// Only use ISSUE_KEY as branch name
        #[arg(short, long)]
        short_branch: bool,
        /// Skip querying Jira for Issue summary
        #[arg(value_name = "ISSUE_KEY")]
        issue_key_input: Option<String>,
    },
    /// Initialise config file(s)
    Init,
    /// Create comment on a Jira Issue
    Comment {
        #[arg(value_name = "COMMENT")]
        comment: String,
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
        comment: Option<String>,
    },
}

impl Commands {
    fn exec(args: Cli) -> Result<(), clap::Error> {
        let cfg = config::Config::load().unwrap();
        let repo = Repository::open();

        match args.command {
            Some(Commands::Branch {
                issue_key_input,
                short_branch,
            }) => {
                let branch_name = if let Some(maybe_issue_key) = issue_key_input {
                    let issue_key = match IssueKey::try_from(maybe_issue_key.clone()) {
                        Ok(i) => i,
                        Err(_) => {
                            eprintln!("Malformed ISSUE_KEY supplied: {}", maybe_issue_key);
                            exit(1);
                        }
                    };

                    if !short_branch {
                        let client = jira::client::JiraAPIClient::build(&cfg);
                        match interactivity::query_issue_details(&client, issue_key.clone()) {
                            Some(i) => i.to_string(),
                            None => {
                                eprintln!("Issue does not exist in Jira: {}", issue_key);
                                exit(1);
                            }
                        }
                    } else {
                        issue_key.to_string()
                    }
                } else {
                    let client = jira::client::JiraAPIClient::build(&cfg);
                    let issues = match interactivity::query_assigned_issues(
                        &client,
                        cfg.default_issue_key,
                    ) {
                        Some(i) => i,
                        None => {
                            eprintln!("Issue does not exist in Jira");
                            exit(1);
                        }
                    };

                    let issue = match interactivity::prompt_user_with_issue_select(issues) {
                        Some(i) => i,
                        None => exit(1),
                    };
                    if short_branch {
                        issue.key.to_string()
                    } else {
                        let branch_name = issue.to_string().replace(' ', "_");
                        match branch_name.len() {
                            n if n > 50 => branch_name.split_at(51).0.to_owned(),
                            _ => branch_name,
                        }
                    }
                };

                // TODO Decide if confirm prompt should be included.
                // let confirm = inquire::Confirm::new(
                //     format!("Create and switch to: {} [y/n]", branch_name).as_str(),
                // )
                // .with_help_message("Defaults to yes")
                // .with_default(true)
                // .prompt_skippable();
                repo.create_branch(branch_name);
            }

            // TODO Implement init flow
            Some(Commands::Init) => println!("Init"),

            // TODO post_comment in jira client
            Some(Commands::Comment {
                comment,
                issue_key_input,
            }) => {
                println!("comment: {}\nissue: {:?}", comment, issue_key_input)
            }

            Some(Commands::Log {
                duration,
                date,
                comment,
            }) => {
                let client = jira::client::JiraAPIClient::build(&cfg);
                let issue = match interactivity::issue_from_branch_or_prompt(&client, &cfg, &repo) {
                    Some(i) => i,
                    None => {
                        eprintln!("No issues returned, consider assigning yourself to some.");
                        if cfg.retry_query_override.is_some() {
                            eprintln!(
                                "Or modify 'retry_query_override' in:\n  {:?}",
                                config::config_file()
                            );
                        } else {
                            eprintln!(
                                "Or try defining 'retry_query_override' in:\n  {:?}",
                                config::config_file()
                            );
                        }
                        exit(1);
                    }
                };

                let wl_duration = match WorklogDuration::try_from(duration.clone()) {
                    Ok(wl) => wl,
                    Err(_) => {
                        eprintln!("Parsing Worklog duration failed: {}", duration);
                        eprintln!(
                            "Ensure duration is a positive number followed by w/d/h\n E.g. 4h"
                        );
                        exit(1);
                    }
                };

                let wl = WorklogAddRequestBody {
                    comment: comment.unwrap_or_default(),
                    started: interactivity::get_date(&cfg, date),
                    time_spent: wl_duration.to_string(),
                };

                match client.post_worklog(issue.key, wl.clone()) {
                    Ok(r) if r.status().is_success() => println!("Worklog created!"),
                    Ok(r) => println!("Worklog creation failed!\n{:?}", r.error_for_status()),
                    Err(e) => {
                        eprintln!("Failed to create worklog: {:?}", wl);
                        eprintln!("{:?}", e);
                        exit(1);
                    }
                }
            }
            None => {
                println!("Planned: Terminal UI or something, dunno")
                // Init if config not found, else help (recommend aliases)
            }
        }
        Ok(())
    }
}

fn main() {
    let args = Cli::parse();

    match Commands::exec(args) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{:?}", e);
            exit(1);
        }
    }
}
