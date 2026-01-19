use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::Provider;
use crate::models::Repository;

pub struct GitHubProvider {
    client: Client,
    base_url: String,
    token: Option<String>,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<GitHubRepo>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    private: bool,
    owner: GitHubOwner,
}

#[derive(Debug, Deserialize)]
struct GitHubOwner {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
}

impl GitHubProvider {
    pub fn new(base_url: String, token: Option<String>, display_name: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
            display_name,
        }
    }

    async fn get_username(&self) -> Result<String> {
        let token = self.token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Authentication required to get username"))?;

        let url = format!("{}/user", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "git-search-cli")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .context("Failed to fetch GitHub user")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API error: {}", response.status());
        }

        let user: GitHubUser = response.json().await?;
        Ok(user.login)
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client
            .get(url)
            .header("User-Agent", "git-search-cli")
            .header("Accept", "application/vnd.github+json");

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request
    }
}

#[async_trait]
impl Provider for GitHubProvider {
    async fn search(&self, query: &str, mine_only: bool, limit: usize) -> Result<Vec<Repository>> {
        let search_query = if mine_only {
            let username = self.get_username().await?;
            format!("{} user:{}", query, username)
        } else {
            query.to_string()
        };

        let url = format!(
            "{}/search/repositories?q={}&per_page={}",
            self.base_url,
            urlencoding::encode(&search_query),
            limit
        );

        let response = self.build_request(&url)
            .send()
            .await
            .context("Failed to search GitHub repositories")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, body);
        }

        let search_response: SearchResponse = response.json().await
            .context("Failed to parse GitHub response")?;

        let display_name = self.display_name.clone();
        let repos = search_response
            .items
            .into_iter()
            .map(|repo| Repository {
                name: repo.name,
                full_name: repo.full_name,
                description: repo.description,
                url: repo.html_url,
                private: repo.private,
                provider: display_name.clone(),
                owner: repo.owner.login,
            })
            .collect();

        Ok(repos)
    }

    fn name(&self) -> &'static str {
        "GitHub"
    }

    fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}
