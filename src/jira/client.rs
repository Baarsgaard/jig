use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use std::convert::From;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct JiraAPIClient {
    pub url: String,
    pub user_email: String,
    pub version: String,

    pub(crate) client: reqwest::Client,
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

    pub fn get_issues(&self, _query: String) -> Vec<String> {
        // self.client
        vec![]
    }

    pub fn my_issues(&self, project: String, text: String) -> Vec<String> {
        let mut query = String::from("assignee = currentuser() ORDER BY updated DESC");

        if !project.is_empty() {
            query = format!("project = {} AND {}", project, query);
        }
        if !text.is_empty() {
            query = format!("text ~ {} AND {}", text, query);
        }

        self.get_issues(query)
    }

    pub fn post_worklog(&self) -> bool {
        true
    }
}
