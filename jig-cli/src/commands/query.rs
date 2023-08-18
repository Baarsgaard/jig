use crate::{config::Config, ExecCommand};
use clap::Args;
use color_eyre::eyre::{eyre, Context, Result};
use inquire::Autocomplete;
use jira::JiraAPIClient;

#[derive(Args, Debug)]
pub struct Query {
    /// Prompt for filter to use a default_query
    #[arg(short = 'f', long = "filter")]
    use_filter: bool,
}

impl ExecCommand for Query {
    fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let query = String::default();

        let issues = match client
            .query_issues(query)
            .wrap_err("First issue query failed")
        {
            Ok(issue_body) => issue_body.issues.unwrap(),
            Err(_) => client
                .query_issues(cfg.retry_query.clone())
                .wrap_err(eyre!("Retry query failed"))?
                .issues
                .unwrap(),
        };

        toml::to_string(&issues).wrap_err("failed exporting issues")
    }
}

#[derive(Clone)] //Default
pub struct IssueQueryBuilder {}

impl IssueQueryBuilder {}

impl Autocomplete for IssueQueryBuilder {
    fn get_suggestions(
        &mut self,
        _input: &str,
    ) -> std::result::Result<Vec<String>, inquire::CustomUserError> {
        todo!()
    }

    fn get_completion(
        &mut self,
        _input: &str,
        _highlighted_suggestion: Option<String>,
    ) -> std::result::Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        todo!()
    }
}
