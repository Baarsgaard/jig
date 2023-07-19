use crate::{config::Config, jira::types::*};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::convert::From;

#[derive(Debug, Clone)]
pub struct JiraAPIClient {
    pub url: String,
    pub version: String,

    pub(crate) client: Client,
    pub(crate) max_results: u32,
}

impl JiraAPIClient {
    pub fn get_headers(cfg: &Config) -> HeaderMap {
        let header_content = HeaderValue::from_static("application/json");

        let mut auth_header_value = if cfg.api_token.is_some() {
            let jira_encoded_auth: String = general_purpose::STANDARD_NO_PAD.encode(format!(
                "{}:{}",
                cfg.user_login.clone().unwrap(),
                cfg.api_token.clone().unwrap(),
            ));
            HeaderValue::from_str(format!("Basic {}", jira_encoded_auth).as_str()).unwrap()
        } else {
            HeaderValue::from_str(format!("Bearer {}", cfg.pat_token.clone().unwrap()).as_str())
                .unwrap()
        };

        auth_header_value.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, header_content.clone());
        headers.insert(CONTENT_TYPE, header_content);
        headers.insert(AUTHORIZATION, auth_header_value);

        headers
    }

    pub fn new(cfg: &Config) -> Result<JiraAPIClient> {
        let mut url = cfg.jira_url.clone();

        if !url.starts_with("http") {
            url = String::from("https://") + &url;
        };

        if url.ends_with('/') {
            url.pop();
        }

        let client = ClientBuilder::new()
            .default_headers(JiraAPIClient::get_headers(cfg))
            .https_only(true)
            .build()?;

        Ok(JiraAPIClient {
            url,
            version: String::from("latest"),
            client,
            max_results: cfg.max_query_results.unwrap_or(50),
        })
    }

    pub fn query_issues(&self, query: String) -> Result<PostIssueQueryResponseBody> {
        let search_url = format!("{}/rest/api/latest/search", self.url.clone());
        let body = PostIssueQueryBody {
            jql: query,
            start_at: 0,
            max_results: self.max_results,
            fields: vec![String::from("summary")],
        };
        let response = self
            .client
            .post(search_url)
            .json(&body)
            .send()
            .context("Post issue query failed")?;

        let query_response_body = response
            .json::<PostIssueQueryResponseBody>()
            .context("Failed parsing issue query response")?;

        if query_response_body.issues.is_empty() {
            return Err(anyhow!("List of issues is empty"));
        }

        Ok(query_response_body)
    }

    pub fn post_worklog(&self, issue_key: &IssueKey, body: PostWorklogBody) -> Result<Response> {
        let worklog_url = format!(
            "{}/rest/api/latest/issue/{}/worklog",
            self.url.clone(),
            issue_key
        );

        let response = self
            .client
            .post(worklog_url)
            .json(&body)
            .send()
            .context("Post worklog request failed")?;
        Ok(response)
    }

    pub fn post_comment(&self, issue_key: &IssueKey, body: PostCommentBody) -> Result<Response> {
        let comment_url = format!(
            "{}/rest/api/latest/issue/{}/comment",
            self.url.clone(),
            issue_key
        );

        let response = self
            .client
            .post(comment_url)
            .json(&body)
            .send()
            .context("Post comment request failed")?;
        Ok(response)
    }

    pub fn get_transitions(&self, issue_key: &IssueKey) -> Result<Vec<Transition>> {
        let transitions_url = format!(
            "{}/rest/api/latest/issue/{}/transitions?expand=transitions.fields",
            self.url.clone(),
            issue_key
        );

        let response = self
            .client
            .get(transitions_url)
            .send()
            .context("Get transitions request failed")?;

        let transition_response_body = response
            .json::<GetTransitionsBody>()
            .context("Failed parsing transition query response")?;

        if transition_response_body.transitions.is_empty() {
            return Err(anyhow!("List of transitions is empty"));
        }

        Ok(transition_response_body.transitions)
    }

    pub fn post_transition(
        &self,
        issue_key: &IssueKey,
        transition: Transition,
    ) -> Result<Response> {
        let transition_url = format!(
            "{}/rest/api/latest/issue/{}/transitions",
            self.url.clone(),
            issue_key
        );

        let body = PostTransitionBody { transition };

        let response = self
            .client
            .post(transition_url)
            .json(&body)
            .send()
            .context("Post transition request failed")?;
        Ok(response)
    }
}
