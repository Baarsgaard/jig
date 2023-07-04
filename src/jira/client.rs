use base64::{engine::general_purpose, Engine as _};
use core::panic;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::convert::From;

use crate::config::Config;
use crate::jira::types::{Issue, IssueQueryBody, IssueQueryResult};

#[derive(Debug, Clone)]
pub struct JiraAPIClient {
    pub url: String,
    pub user_email: String,
    pub version: String,

    pub(crate) client: reqwest::Client,
    pub(crate) max_results: u32,
}

impl JiraAPIClient {
    pub fn get_headers(cfg: Config) -> HeaderMap {
        let header_content = HeaderValue::from_static("application/json");

        let jira_encoded_auth: String = general_purpose::STANDARD_NO_PAD
            .encode(format!("{}:{}", cfg.user_email, cfg.api_token));

        let mut header_basic_auth_token =
            HeaderValue::from_str(format!("Basic {}", jira_encoded_auth).as_str()).unwrap();
        header_basic_auth_token.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, header_content.clone());
        headers.insert(CONTENT_TYPE, header_content);
        headers.insert(AUTHORIZATION, header_basic_auth_token);

        headers
    }

    #[tokio::main]
    pub async fn fetch_issues(&self, query: String) -> Result<IssueQueryResult, reqwest::Error> {
        let search_url = self.url.clone() + "/rest/api/latest/search";
        let body = IssueQueryBody {
            jql: query,
            start_at: 0,
            max_results: self.max_results,
            fields: vec![String::from("summary")],
        };
        let response = self.client.post(search_url).json(&body).send().await?;

        response.json().await
    }

    pub fn my_issues(&self, project: String) -> Vec<Issue> {
        let mut query = String::from("assignee = currentuser() ORDER BY updated DESC");

        if !project.is_empty() {
            query = format!("project = {} AND {}", project, query);
        }

        match self.fetch_issues(query) {
            Ok(issue_query_result) => issue_query_result.issues,
            Err(e) => {
                eprint!("{}", e);
                Vec::new()
            }
        }
    }

    pub fn post_worklog(&self) -> bool {
        true
    }
}
