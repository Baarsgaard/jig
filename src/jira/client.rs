use base64::{engine::general_purpose, Engine as _};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Error;
use std::convert::From;

use crate::config::Config;
use crate::jira::types::{IssueQueryRequestBody, IssueQueryResponseResult};

use super::types::{IssueKey, WorklogAddRequestBody};

#[derive(Debug, Clone)]
pub struct JiraAPIClient {
    pub url: String,
    pub user_login: String,
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
                cfg.user_login,
                cfg.api_token.clone().unwrap()
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

    pub fn build(cfg: &Config) -> JiraAPIClient {
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
            .build()
            .expect("Unable to instantiate request client");

        JiraAPIClient {
            url,
            user_login: cfg.user_login.clone(),
            version: String::from("latest"),
            client,
            max_results: cfg.max_query_results.unwrap_or(15),
        }
    }

    pub fn query_issues(&self, query: String) -> Result<IssueQueryResponseResult, Error> {
        let search_url = self.url.clone() + "/rest/api/latest/search";
        let body = IssueQueryRequestBody {
            jql: query,
            start_at: 0,
            max_results: self.max_results,
            fields: vec![String::from("summary")],
        };
        let response = self.client.post(search_url).json(&body).send()?;

        response.json()
    }

    pub fn post_worklog(
        &self,
        issue_key: IssueKey,
        body: WorklogAddRequestBody,
    ) -> Result<Response, Error> {
        let worklog_url =
            self.url.clone() + format!("/rest/api/latest/issue/{}/worklog", issue_key).as_str();

        self.client.post(worklog_url).json(&body).send()
    }
}
