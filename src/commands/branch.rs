use crate::{
    config::Config,
    interactivity::{issue_from_branch_or_prompt, query_issue_details},
    repo::Repository,
};
use clap::{Args, ValueHint};
use color_eyre::{
    Section,
    eyre::{Result, WrapErr, eyre},
};
use inquire::Select;
use jira::{JiraAPIClient, models::IssueKey};

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Branch {
    /// Add string to the end of branch name.
    /// SUFFIX is subject to Git sanitization rules
    #[arg(short, long, value_name = "SUFFIX", value_hint = ValueHint::Unknown)]
    append: Option<String>,

    /// Overwrite Issue summary and supply a branch name
    #[arg(short, long, visible_aliases = ["n", "name"], value_name = "OVERWRITE", value_hint = ValueHint::Unknown)]
    overwrite: Option<String>,

    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Use ISSUE_KEY as the branch name
    /// Inverts 'always_short_branch_names' setting
    #[arg(short = 's', long = "short")]
    short_name: bool,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Branch {
    async fn exec(self, cfg: &Config) -> Result<String> {
        if matches!(
            (
                self.short_name,
                self.append.is_some(),
                self.overwrite.is_some(),
            ),
            (true, true, _) | (true, _, true) | (_, true, true)
        ) {
            return Err(eyre!("Multiple branch name modifiers specified"))
                .with_suggestion(|| "Avoid specifying Append, Overwrite, and short together");
        }

        let repo = Repository::open().wrap_err("Failed to open repository")?;
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let issue = if let Some(maybe_issue_key) = self.issue_key_input {
            let issue_key = IssueKey::try_from(maybe_issue_key)?;

            query_issue_details(&client, issue_key).await?
        } else {
            issue_from_branch_or_prompt(&client, cfg, String::default(), self.use_filter).await?
        };

        // Get existing branches
        let branches = repo
            .get_existing_branches(&issue.key.to_string())
            .context("Failed to read branch names")?;

        // Prompt user if branches exist or fall back to selected issue.
        let opt_existing_branch = if !branches.is_empty() {
            Select::new("Checkout existing branch?", branches)
                .with_help_message("Skip by pressing the escape key")
                .prompt_skippable()
                .context("Failed to open prompt with existing branches")?
        } else {
            None
        };

        let branch_name = if let Some(branch_name) = opt_existing_branch.clone() {
            branch_name
        } else if self.short_name {
            issue.key.to_string()
        } else if let Some(overwritten_name) = self.overwrite {
            Repository::sanitize_branch_name(&format!("{} {}", issue.key, overwritten_name))
        } else {
            Repository::branch_name_from_issue(&issue, self.append)?
        };

        repo.checkout_branch(&branch_name, opt_existing_branch.is_none())
    }
}
