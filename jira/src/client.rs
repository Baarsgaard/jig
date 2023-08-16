use crate::types::*;
use base64::{engine::general_purpose, Engine as _};
use color_eyre::eyre::{eyre, Result, WrapErr};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Url;
use std::convert::From;

///
#[derive(Debug, Clone)]
pub struct JiraClientConfig {
    pub credential: Credential,
    pub max_query_results: u32,
    pub url: String,
}

/// Supported Authentication methods
#[derive(Debug, Clone)]
pub enum Credential {
    /// Anonymous
    /// Omit Authorization header
    Anonymous,
    /// User email/username and token
    /// Authorization: Basic <b64 login:token>
    ApiToken { login: String, token: String },
    /// Personal Access Token
    /// Authorization: Bearer <PAT>
    PersonalAccessToken(String),
}

/// Reusable client for interfacing with Jira
#[derive(Debug, Clone)]
pub struct JiraAPIClient {
    pub url: Url,
    pub version: String,

    pub(crate) client: Client,
    pub(crate) max_results: u32,
}

impl JiraAPIClient {
    fn build_headers(credentials: &Credential) -> HeaderMap {
        let header_content = HeaderValue::from_static("application/json");

        let auth_header = match credentials {
            Credential::Anonymous => None,
            Credential::ApiToken {
                login: user_login,
                token: api_token,
            } => {
                let jira_encoded_auth = general_purpose::STANDARD_NO_PAD
                    .encode(format!("{}:{}", user_login, api_token,));
                Some(HeaderValue::from_str(&format!("Basic {}", jira_encoded_auth)).unwrap())
            }
            Credential::PersonalAccessToken(token) => {
                Some(HeaderValue::from_str(&format!("Bearer {}", token)).unwrap())
            }
        };

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, header_content.clone());
        headers.insert(CONTENT_TYPE, header_content);

        if let Some(mut auth_header_value) = auth_header {
            auth_header_value.set_sensitive(true);
            headers.insert(AUTHORIZATION, auth_header_value);
        }

        headers
    }

    pub fn new(cfg: &JiraClientConfig) -> Result<JiraAPIClient> {
        let client = ClientBuilder::new()
            .default_headers(JiraAPIClient::build_headers(&cfg.credential))
            .https_only(true)
            .build()?;

        Ok(JiraAPIClient {
            url: Url::parse(cfg.url.as_str()).wrap_err("Unable to construct JiraApiClient.url")?,
            version: String::from("latest"),
            client,
            max_results: cfg.max_query_results,
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
            .wrap_err("Post issue query failed")?;

        let query_response_body = response
            .json::<PostIssueQueryResponseBody>()
            .wrap_err("Failed parsing issue query response")?;

        let _ = match query_response_body.issues.clone() {
            Some(i) if i.is_empty() => Err(eyre!("List of issues is empty"))?,
            Some(i) => i,
            None => Err(eyre!("List of issues is empty"))?,
        };

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
            .wrap_err("Post worklog request failed")?;
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
            .wrap_err("Post comment request failed")?;
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
            .wrap_err("Get transitions request failed")?;

        let transition_response_body = response
            .json::<GetTransitionsBody>()
            .wrap_err("Failed parsing transition query response")?;

        if transition_response_body.transitions.is_empty() {
            return Err(eyre!("List of transitions is empty"));
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
            .wrap_err("Post transition request failed")?;
        Ok(response)
    }

    pub fn search_filters(&self, filter: Option<String>) -> Result<GetFilterResponseBody> {
        let mut search_url = format!(
            "{}/rest/api/latest/filter/search?expand=jql&maxResults={}",
            self.url.clone(),
            self.max_results
        );

        if let Some(filter) = filter {
            search_url.push_str(&format!("&filterName={}", filter));
        }

        let response = self
            .client
            .get(search_url)
            .send()
            .wrap_err("Fetching issue filters failed")?;

        response
            .json::<GetFilterResponseBody>()
            .wrap_err("Failed parsing filter search response")
    }
}
