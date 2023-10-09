use crate::types::*;
use base64::{engine::general_purpose, Engine as _};
use color_eyre::eyre::{eyre, Result, WrapErr};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Url;
use std::convert::From;
use std::time::Duration;

/// Used to configure JiraApiClient upon instantiation
#[derive(Debug, Clone)]
pub struct JiraClientConfig {
    pub credential: Credential,
    pub max_query_results: u32,
    pub url: String,
    pub timeout: u64,
    pub tls_accept_invalid_certs: bool,
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

    /// Instantiate a reusable API client.
    ///
    /// ```rust
    /// use jira::types::*;
    /// use jira::{Credential, JiraClientConfig, JiraAPIClient};
    ///
    /// let anon = Credential::Anonymous;
    ///
    /// // let credential = Credential::PersonalAccessToken("xxxxxxx".to_string())
    ///
    /// // let api_token = Credential::ApiToken {
    /// //     login: "user@example.com".to_string(),
    /// //     token: "xxxxxxx".to_string(),
    /// // };
    ///
    /// let jira_cfg = JiraClientConfig {
    ///     credential: anon,
    ///     max_query_results: 50u32,
    ///     url: "https://domain.atlassian.net".to_string(),
    ///     timeout: 10u64,
    ///     tls_accept_invalid_certs: false,
    /// };
    ///
    /// let client = JiraAPIClient::new(&jira_cfg).unwrap();
    /// ```
    pub fn new(cfg: &JiraClientConfig) -> Result<JiraAPIClient> {
        let client = ClientBuilder::new()
            .default_headers(JiraAPIClient::build_headers(&cfg.credential))
            .danger_accept_invalid_certs(cfg.tls_accept_invalid_certs)
            .https_only(true)
            .timeout(Some(Duration::from_secs(cfg.timeout)))
            .build()?;

        Ok(JiraAPIClient {
            url: Url::parse(cfg.url.as_str()).wrap_err("Unable to construct JiraApiClient.url")?,
            version: String::from("latest"),
            client,
            max_results: cfg.max_query_results,
        })
    }

    pub fn query_issues(&self, query: &String) -> Result<PostIssueQueryResponseBody> {
        let search_url = self.url.join("/rest/api/latest/search")?;
        let body = PostIssueQueryBody {
            jql: query.to_owned(),
            start_at: 0,
            max_results: self.max_results,
            fields: vec![String::from("summary")],
        };

        let response = self
            .client
            .post(search_url)
            .json(&body)
            .send()
            .wrap_err("POST issue query failed")?;

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
        let worklog_url = self
            .url
            .join(format!("/rest/api/latest/issue/{}/worklog", issue_key).as_str())?;

        // If any pattern matches, do not prompt.
        if matches!(
            (body.time_spent.is_some(), body.time_spent_seconds.is_some()),
            (false, false) | (true, true)
        ) {
            return Err(eyre!(
                "Malformed body: time_spent and time_spent_seconds are botn 'Some()' or 'None'"
            ));
        }

        let response = self
            .client
            .post(worklog_url)
            .json(&body)
            .send()
            .wrap_err("POST worklog request failed")?;
        Ok(response)
    }

    pub fn post_comment(&self, issue_key: &IssueKey, body: PostCommentBody) -> Result<Response> {
        let comment_url = self
            .url
            .join(format!("/rest/api/latest/issue/{}/comment", issue_key).as_str())?;

        let response = self
            .client
            .post(comment_url)
            .json(&body)
            .send()
            .wrap_err("POST comment request failed")?;
        Ok(response)
    }

    pub fn get_transitions(&self, issue_key: &IssueKey) -> Result<Vec<Transition>> {
        let transitions_url = self.url.join(
            format!(
                "/rest/api/latest/issue/{}/transitions?expand=transitions.fields",
                issue_key
            )
            .as_str(),
        )?;

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
        let transition_url = self
            .url
            .join(format!("/rest/api/latest/issue/{}/transitions", issue_key).as_str())?;

        let body = PostTransitionBody { transition };

        let response = self
            .client
            .post(transition_url)
            .json(&body)
            .send()
            .wrap_err("POST transition request failed")?;
        Ok(response)
    }

    pub fn get_assignable_users(&self, params: &GetAssignableUserParams) -> Result<Vec<User>> {
        let mut users_url = self.url.join("/rest/api/latest/user/assignable/search")?;
        let mut query: String = format!("maxResults={}", params.max_results.unwrap_or(1000));

        if params.project.is_none() && params.issue_key.is_none() {
            Err(eyre!(
                "Both project and issue_key are None, define either to query for assignable users."
            ))?
        }

        if let Some(issue_key) = params.issue_key.clone() {
            query.push_str(format!("&issueKey={}", issue_key).as_str());
        }
        if let Some(username) = params.username.clone() {
            #[cfg(feature = "cloud")]
            query.push_str(format!("&query={}", username).as_str());
            #[cfg(not(feature = "cloud"))]
            query.push_str(format!("&username={}", username).as_str());
        }
        if let Some(project) = params.project.clone() {
            query.push_str(format!("&project={}", project).as_str());
        }

        users_url.set_query(Some(query.as_str()));

        let response = self.client.get(users_url).send().wrap_err(format!(
            "Unable to fetch assignable users for issue: {:?}",
            params
        ))?;

        response
            .json::<Vec<User>>()
            .wrap_err("Failed parsing assignable users list")
    }

    pub fn post_assign_user(&self, issue_key: &IssueKey, user: &User) -> Result<Response> {
        let assign_url = self
            .url
            .join(format!("/rest/api/latest/issue/{}/assignee", issue_key).as_str())?;

        let body = PostAssignBody::from(user.clone());

        let response = self
            .client
            .put(assign_url)
            .json(&body)
            .send()
            .wrap_err("PUT assign user request failed")?;
        Ok(response)
    }

    /// cloud:  user.account_id
    /// server: user.name
    pub fn get_user(&self, user: String) -> Result<User> {
        let user_url = self.url.join("/rest/api/latest/user")?;

        let key = match cfg!(feature = "cloud") {
            true => "accountId",
            false => "username",
        };

        let response = self
            .client
            .get(user_url)
            .query(&[(key, user)])
            .send()
            .wrap_err("GET User request failed")?;

        Ok(response.json::<User>()?)
    }

    #[cfg(feature = "cloud")]
    pub fn search_filters(&self, filter: Option<String>) -> Result<GetFilterResponseBody> {
        let mut search_url = self.url.join("/rest/api/latest/filter/search")?;
        let query = if let Some(filter) = filter {
            format!(
                "expand=jql&maxResults={}&filterName={}",
                self.max_results, filter
            )
        } else {
            format!("expand=jql&maxResults={}", self.max_results)
        };

        search_url.set_query(Some(query.as_str()));

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
