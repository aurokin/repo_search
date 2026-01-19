pub mod bitbucket;
pub mod github;
pub mod gitlab;

use anyhow::Result;
use async_trait::async_trait;

use crate::models::Repository;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn search(&self, query: &str, mine_only: bool, limit: usize) -> Result<Vec<Repository>>;
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    #[allow(dead_code)]
    fn is_authenticated(&self) -> bool;
}

pub use bitbucket::BitbucketProvider;
pub use github::GitHubProvider;
pub use gitlab::GitLabProvider;
