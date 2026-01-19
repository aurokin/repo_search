use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use super::Provider;
use crate::models::Repository;

pub struct BitbucketProvider {
    client: Client,
    base_url: String,
    token: Option<String>,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketResponse {
    values: Vec<BitbucketRepo>,
}

#[derive(Debug, Deserialize)]
struct BitbucketRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    is_private: bool,
    links: BitbucketLinks,
    owner: BitbucketOwner,
}

#[derive(Debug, Deserialize)]
struct BitbucketLinks {
    html: BitbucketLink,
}

#[derive(Debug, Deserialize)]
struct BitbucketLink {
    href: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketOwner {
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct BitbucketUser {
    username: String,
}

impl BitbucketProvider {
    pub fn new(base_url: String, token: Option<String>, display_name: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
            display_name,
        }
    }

    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url).header("User-Agent", "repo_search_cli");

        if let Some(token) = &self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        request
    }

    async fn get_username(&self) -> Result<String> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Authentication required to get username"))?;

        let url = format!("{}/user", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "repo_search_cli")
            .send()
            .await
            .context("Failed to fetch Bitbucket user")?;

        if !response.status().is_success() {
            anyhow::bail!("Bitbucket API error: {}", response.status());
        }

        let user: BitbucketUser = response.json().await?;
        Ok(user.username)
    }
}

#[async_trait]
impl Provider for BitbucketProvider {
    async fn search(&self, query: &str, mine_only: bool, limit: usize) -> Result<Vec<Repository>> {
        // Bitbucket requires authentication for searching all repositories
        // Without auth, we can only search within a specific user's repos
        if !mine_only && self.token.is_none() {
            anyhow::bail!("Bitbucket requires authentication to search all repositories. Set BITBUCKET_TOKEN or use --mine flag.");
        }

        let url = if mine_only || self.token.is_some() {
            let username = self.get_username().await?;
            format!(
                "{}/repositories/{}?q=name~\"{}\"&pagelen={}",
                self.base_url,
                username,
                urlencoding::encode(query),
                limit
            )
        } else {
            format!(
                "{}/repositories?q=name~\"{}\"&pagelen={}",
                self.base_url,
                urlencoding::encode(query),
                limit
            )
        };

        let response = self
            .build_request(&url)
            .send()
            .await
            .context("Failed to search Bitbucket repositories")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Bitbucket API error ({}): {}", status, body);
        }

        let bitbucket_response: BitbucketResponse = response
            .json()
            .await
            .context("Failed to parse Bitbucket response")?;

        let display_name = self.display_name.clone();
        let repos = bitbucket_response
            .values
            .into_iter()
            .map(|repo| Repository {
                name: repo.name,
                full_name: repo.full_name,
                description: repo.description,
                url: repo.links.html.href,
                private: repo.is_private,
                provider: display_name.clone(),
                owner: repo.owner.display_name,
            })
            .collect();

        Ok(repos)
    }

    fn name(&self) -> &'static str {
        "Bitbucket"
    }

    fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}
