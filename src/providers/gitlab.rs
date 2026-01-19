use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::Provider;
use crate::models::Repository;

pub struct GitLabProvider {
    client: Client,
    base_url: String,
    token: Option<String>,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct GitLabProject {
    name: String,
    path_with_namespace: String,
    description: Option<String>,
    web_url: String,
    visibility: String,
    namespace: GitLabNamespace,
}

#[derive(Debug, Deserialize)]
struct GitLabNamespace {
    name: String,
}

impl GitLabProvider {
    pub fn new(base_url: String, token: Option<String>, display_name: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
            display_name,
        }
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client
            .get(url)
            .header("User-Agent", "git-search-cli");

        if let Some(token) = &self.token {
            request = request.header("PRIVATE-TOKEN", token);
        }

        request
    }
}

#[async_trait]
impl Provider for GitLabProvider {
    async fn search(&self, query: &str, mine_only: bool, limit: usize) -> Result<Vec<Repository>> {
        let mut url = format!(
            "{}/api/v4/projects?search={}&per_page={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        if mine_only {
            url.push_str("&owned=true");
        }

        let response = self.build_request(&url)
            .send()
            .await
            .context("Failed to search GitLab projects")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitLab API error ({}): {}", status, body);
        }

        let projects: Vec<GitLabProject> = response.json().await
            .context("Failed to parse GitLab response")?;

        let display_name = self.display_name.clone();
        let repos = projects
            .into_iter()
            .map(|project| Repository {
                name: project.name,
                full_name: project.path_with_namespace.clone(),
                description: project.description,
                url: project.web_url,
                private: project.visibility != "public",
                provider: display_name.clone(),
                owner: project.namespace.name,
            })
            .collect();

        Ok(repos)
    }

    fn name(&self) -> &'static str {
        "GitLab"
    }

    fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}
