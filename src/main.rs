mod cli;
mod config;
mod models;
mod output;
mod providers;

use std::collections::HashSet;

use anyhow::Result;
use config::{Config, ProviderType, ResolvedProvider};
use models::Repository;
use providers::{BitbucketProvider, GitHubProvider, GitLabProvider, Provider};

const DEFAULT_LIMIT: usize = 10;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse();
    let config = Config::load()?;

    // Handle --list-providers flag
    if args.list_providers {
        println!("Configured providers:");
        for name in config.provider_names() {
            if let Some(resolved) = config.resolve_provider(&name) {
                let type_str = match resolved.provider_type {
                    ProviderType::Github => "github",
                    ProviderType::Gitlab => "gitlab",
                    ProviderType::Bitbucket => "bitbucket",
                };
                let auth = if resolved.token.is_some() {
                    " (authenticated)"
                } else {
                    ""
                };
                println!("  {} [{}] -> {}{}", name, type_str, resolved.url, auth);
            }
        }
        return Ok(());
    }

    if args.mine && args.owner.is_some() {
        eprintln!("Error: --owner and --mine cannot be used together");
        std::process::exit(1);
    }

    // Resolve limit: CLI > config > default
    let limit = args
        .limit
        .or(config.defaults.limit)
        .unwrap_or(DEFAULT_LIMIT);

    // Require query for search
    let query = match args.query {
        Some(q) => q,
        None => {
            eprintln!("Error: Search query is required");
            eprintln!("Usage: repo_search <QUERY>");
            std::process::exit(1);
        }
    };

    // Resolve which providers to search
    let provider_names = resolve_provider_names(&args.provider, &config);

    // Resolve provider configurations
    let mut resolved_providers: Vec<ResolvedProvider> = Vec::new();
    for name in &provider_names {
        match config.resolve_provider(name) {
            Some(mut resolved) => {
                // Apply URL override from CLI if provided
                if let Some(ref url) = args.url {
                    resolved.url = url.clone();
                }
                resolved_providers.push(resolved);
            }
            None => {
                eprintln!("Warning: Unknown provider '{}', skipping", name);
            }
        }
    }

    if resolved_providers.is_empty() {
        eprintln!("Error: No valid providers to search");
        std::process::exit(1);
    }

    // Execute searches
    let (repos, errors) = execute_searches(
        &resolved_providers,
        &query,
        args.mine,
        args.owner.as_deref(),
        limit,
    )
    .await;

    // Print warnings
    if !errors.is_empty() && !args.json {
        for error in &errors {
            eprintln!("Warning: {}", error);
        }
        if !repos.is_empty() {
            eprintln!();
        }
    }

    output::print_results(repos, args.json);

    Ok(())
}

fn resolve_provider_names(cli_providers: &[String], config: &Config) -> Vec<String> {
    if !cli_providers.is_empty() {
        // Expand "all" to all configured providers
        let mut names = HashSet::new();
        for p in cli_providers {
            if p.eq_ignore_ascii_case("all") {
                for name in config.provider_names() {
                    names.insert(name);
                }
            } else {
                names.insert(p.clone());
            }
        }
        return names.into_iter().collect();
    }

    // Use config defaults
    config.default_providers()
}

async fn execute_searches(
    providers: &[ResolvedProvider],
    query: &str,
    mine_only: bool,
    owner: Option<&str>,
    limit: usize,
) -> (Vec<Repository>, Vec<String>) {
    use tokio::task::JoinSet;

    let mut join_set = JoinSet::new();

    for provider in providers {
        let name = provider.name.clone();
        let url = provider.url.clone();
        let token = provider.token.clone();
        let provider_type = provider.provider_type;
        let query = query.to_string();
        let owner = owner.map(|value| value.to_string());

        join_set.spawn(async move {
            let result: Result<Vec<Repository>> = match provider_type {
                ProviderType::Github => {
                    let p = GitHubProvider::new(url, token, name.clone());
                    p.search(&query, mine_only, owner.as_deref(), limit).await
                }
                ProviderType::Gitlab => {
                    let p = GitLabProvider::new(url, token, name.clone());
                    p.search(&query, mine_only, owner.as_deref(), limit).await
                }
                ProviderType::Bitbucket => {
                    let p = BitbucketProvider::new(url, token, name.clone());
                    p.search(&query, mine_only, owner.as_deref(), limit).await
                }
            };
            (name, result)
        });
    }

    let mut all_repos = Vec::new();
    let mut errors = Vec::new();

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok((_name, Ok(repos))) => {
                all_repos.extend(repos);
            }
            Ok((name, Err(e))) => {
                errors.push(format!("{}: {}", name, e));
            }
            Err(e) => {
                errors.push(format!("Task error: {}", e));
            }
        }
    }

    (all_repos, errors)
}
