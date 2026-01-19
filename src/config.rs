use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Github,
    Gitlab,
    Bitbucket,
}

impl ProviderType {
    pub fn default_url(&self) -> &'static str {
        match self {
            ProviderType::Github => "https://api.github.com",
            ProviderType::Gitlab => "https://gitlab.com",
            ProviderType::Bitbucket => "https://api.bitbucket.org/2.0",
        }
    }

    /// Try to infer type from provider name (for backwards compatibility)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "github" => Some(ProviderType::Github),
            "gitlab" => Some(ProviderType::Gitlab),
            "bitbucket" => Some(ProviderType::Bitbucket),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Named provider instances
    #[serde(default)]
    pub providers: HashMap<String, ProviderEntry>,

    // Legacy top-level provider configs (backwards compatibility)
    #[serde(default)]
    github: Option<LegacyProviderConfig>,
    #[serde(default)]
    gitlab: Option<LegacyProviderConfig>,
    #[serde(default)]
    bitbucket: Option<LegacyProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultsConfig {
    /// Default providers to search (e.g., ["github", "gitlab", "work-bb"])
    pub providers: Option<Vec<String>>,
    /// Default result limit per provider
    pub limit: Option<usize>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            providers: None,
            limit: None,
        }
    }
}

/// A named provider entry in the config
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderEntry {
    /// Provider type (github, gitlab, bitbucket)
    /// Optional for entries named "github", "gitlab", or "bitbucket"
    #[serde(rename = "type")]
    pub provider_type: Option<ProviderType>,
    pub token: Option<String>,
    pub url: Option<String>,
}

/// Legacy provider config (top-level [github], [gitlab], [bitbucket])
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LegacyProviderConfig {
    pub token: Option<String>,
    pub url: Option<String>,
}

/// Resolved provider configuration ready for use
#[derive(Debug, Clone)]
pub struct ResolvedProvider {
    pub name: String,
    pub provider_type: ProviderType,
    pub token: Option<String>,
    pub url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();
        config.migrate_legacy_providers();
        Ok(config)
    }

    fn load_from_file() -> Result<Self> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("repo_search").join("config.toml"))
    }

    /// Migrate legacy top-level provider configs to the providers map
    fn migrate_legacy_providers(&mut self) {
        if let Some(legacy) = self.github.take() {
            self.providers
                .entry("github".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Github),
                    token: legacy.token,
                    url: legacy.url,
                });
        }
        if let Some(legacy) = self.gitlab.take() {
            self.providers
                .entry("gitlab".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Gitlab),
                    token: legacy.token,
                    url: legacy.url,
                });
        }
        if let Some(legacy) = self.bitbucket.take() {
            self.providers
                .entry("bitbucket".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Bitbucket),
                    token: legacy.token,
                    url: legacy.url,
                });
        }
    }

    fn apply_env_overrides(&mut self) {
        // Apply env overrides to named providers or create them
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            self.providers
                .entry("github".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Github),
                    token: None,
                    url: None,
                })
                .token = Some(token);
        }
        if let Ok(url) = env::var("GITHUB_URL") {
            self.providers
                .entry("github".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Github),
                    token: None,
                    url: None,
                })
                .url = Some(url);
        }

        if let Ok(token) = env::var("GITLAB_TOKEN") {
            self.providers
                .entry("gitlab".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Gitlab),
                    token: None,
                    url: None,
                })
                .token = Some(token);
        }
        if let Ok(url) = env::var("GITLAB_URL") {
            self.providers
                .entry("gitlab".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Gitlab),
                    token: None,
                    url: None,
                })
                .url = Some(url);
        }

        if let Ok(token) = env::var("BITBUCKET_TOKEN") {
            self.providers
                .entry("bitbucket".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Bitbucket),
                    token: None,
                    url: None,
                })
                .token = Some(token);
        }
        if let Ok(url) = env::var("BITBUCKET_URL") {
            self.providers
                .entry("bitbucket".to_string())
                .or_insert(ProviderEntry {
                    provider_type: Some(ProviderType::Bitbucket),
                    token: None,
                    url: None,
                })
                .url = Some(url);
        }
    }

    /// Resolve a provider by name, returning its full configuration
    pub fn resolve_provider(&self, name: &str) -> Option<ResolvedProvider> {
        // Check if it's a configured provider
        if let Some(entry) = self.providers.get(name) {
            let provider_type = entry
                .provider_type
                .or_else(|| ProviderType::from_name(name))?;

            return Some(ResolvedProvider {
                name: name.to_string(),
                provider_type,
                token: entry.token.clone(),
                url: entry
                    .url
                    .clone()
                    .unwrap_or_else(|| provider_type.default_url().to_string()),
            });
        }

        // Check if it's a built-in provider name without config
        if let Some(provider_type) = ProviderType::from_name(name) {
            return Some(ResolvedProvider {
                name: name.to_string(),
                provider_type,
                token: None,
                url: provider_type.default_url().to_string(),
            });
        }

        None
    }

    /// Get all configured provider names
    pub fn provider_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.providers.keys().cloned().collect();
        // Add built-in providers if not already present
        for builtin in ["github", "gitlab", "bitbucket"] {
            if !names.contains(&builtin.to_string()) {
                names.push(builtin.to_string());
            }
        }
        names.sort();
        names
    }

    /// Get default provider names to search
    pub fn default_providers(&self) -> Vec<String> {
        self.defaults.providers.clone().unwrap_or_else(|| {
            vec![
                "github".to_string(),
                "gitlab".to_string(),
                "bitbucket".to_string(),
            ]
        })
    }

    /// Parse config from a TOML string (for testing)
    #[cfg(test)]
    pub fn from_toml(content: &str) -> Result<Self> {
        let mut config: Config = toml::from_str(content)?;
        config.migrate_legacy_providers();
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_name() {
        assert_eq!(
            ProviderType::from_name("github"),
            Some(ProviderType::Github)
        );
        assert_eq!(
            ProviderType::from_name("GitHub"),
            Some(ProviderType::Github)
        );
        assert_eq!(
            ProviderType::from_name("GITHUB"),
            Some(ProviderType::Github)
        );
        assert_eq!(
            ProviderType::from_name("gitlab"),
            Some(ProviderType::Gitlab)
        );
        assert_eq!(
            ProviderType::from_name("bitbucket"),
            Some(ProviderType::Bitbucket)
        );
        assert_eq!(ProviderType::from_name("unknown"), None);
        assert_eq!(ProviderType::from_name("work-gitlab"), None);
    }

    #[test]
    fn test_provider_type_default_url() {
        assert_eq!(ProviderType::Github.default_url(), "https://api.github.com");
        assert_eq!(ProviderType::Gitlab.default_url(), "https://gitlab.com");
        assert_eq!(
            ProviderType::Bitbucket.default_url(),
            "https://api.bitbucket.org/2.0"
        );
    }

    #[test]
    fn test_parse_empty_config() {
        let config = Config::from_toml("").unwrap();
        assert!(config.defaults.providers.is_none());
        assert!(config.defaults.limit.is_none());
        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_parse_defaults() {
        let toml = r#"
            [defaults]
            providers = ["github", "gitlab"]
            limit = 20
        "#;
        let config = Config::from_toml(toml).unwrap();
        assert_eq!(
            config.defaults.providers,
            Some(vec!["github".to_string(), "gitlab".to_string()])
        );
        assert_eq!(config.defaults.limit, Some(20));
    }

    #[test]
    fn test_parse_providers() {
        let toml = r#"
            [providers.github]
            token = "ghp_test"

            [providers.work-gitlab]
            type = "gitlab"
            url = "https://gitlab.work.com"
            token = "work-token"
        "#;
        let config = Config::from_toml(toml).unwrap();

        assert!(config.providers.contains_key("github"));
        assert!(config.providers.contains_key("work-gitlab"));

        let github = &config.providers["github"];
        assert_eq!(github.token, Some("ghp_test".to_string()));

        let work_gitlab = &config.providers["work-gitlab"];
        assert_eq!(work_gitlab.provider_type, Some(ProviderType::Gitlab));
        assert_eq!(work_gitlab.url, Some("https://gitlab.work.com".to_string()));
        assert_eq!(work_gitlab.token, Some("work-token".to_string()));
    }

    #[test]
    fn test_legacy_config_migration() {
        let toml = r#"
            [github]
            token = "legacy-github-token"
            url = "https://github.enterprise.com"

            [gitlab]
            token = "legacy-gitlab-token"
        "#;
        let config = Config::from_toml(toml).unwrap();

        // Legacy configs should be migrated to providers
        assert!(config.providers.contains_key("github"));
        assert!(config.providers.contains_key("gitlab"));

        let github = &config.providers["github"];
        assert_eq!(github.token, Some("legacy-github-token".to_string()));
        assert_eq!(
            github.url,
            Some("https://github.enterprise.com".to_string())
        );
    }

    #[test]
    fn test_resolve_builtin_provider() {
        let config = Config::from_toml("").unwrap();

        let github = config.resolve_provider("github").unwrap();
        assert_eq!(github.name, "github");
        assert_eq!(github.provider_type, ProviderType::Github);
        assert_eq!(github.url, "https://api.github.com");
        assert!(github.token.is_none());
    }

    #[test]
    fn test_resolve_configured_provider() {
        let toml = r#"
            [providers.github]
            token = "my-token"
            url = "https://api.github.enterprise.com"
        "#;
        let config = Config::from_toml(toml).unwrap();

        let github = config.resolve_provider("github").unwrap();
        assert_eq!(github.token, Some("my-token".to_string()));
        assert_eq!(github.url, "https://api.github.enterprise.com");
    }

    #[test]
    fn test_resolve_custom_provider() {
        let toml = r#"
            [providers.work-gitlab]
            type = "gitlab"
            url = "https://gitlab.work.com"
            token = "work-token"
        "#;
        let config = Config::from_toml(toml).unwrap();

        let provider = config.resolve_provider("work-gitlab").unwrap();
        assert_eq!(provider.name, "work-gitlab");
        assert_eq!(provider.provider_type, ProviderType::Gitlab);
        assert_eq!(provider.url, "https://gitlab.work.com");
        assert_eq!(provider.token, Some("work-token".to_string()));
    }

    #[test]
    fn test_resolve_custom_provider_without_type_fails() {
        let toml = r#"
            [providers.my-custom]
            url = "https://custom.com"
            token = "token"
        "#;
        let config = Config::from_toml(toml).unwrap();

        // Should return None because type cannot be inferred from "my-custom"
        assert!(config.resolve_provider("my-custom").is_none());
    }

    #[test]
    fn test_resolve_unknown_provider() {
        let config = Config::from_toml("").unwrap();
        assert!(config.resolve_provider("unknown-provider").is_none());
    }

    #[test]
    fn test_provider_names_includes_builtins() {
        let config = Config::from_toml("").unwrap();
        let names = config.provider_names();

        assert!(names.contains(&"github".to_string()));
        assert!(names.contains(&"gitlab".to_string()));
        assert!(names.contains(&"bitbucket".to_string()));
    }

    #[test]
    fn test_provider_names_includes_custom() {
        let toml = r#"
            [providers.work-gitlab]
            type = "gitlab"
            url = "https://gitlab.work.com"
        "#;
        let config = Config::from_toml(toml).unwrap();
        let names = config.provider_names();

        assert!(names.contains(&"work-gitlab".to_string()));
        assert!(names.contains(&"github".to_string())); // builtins still included
    }

    #[test]
    fn test_default_providers_from_config() {
        let toml = r#"
            [defaults]
            providers = ["github", "work-gitlab"]
        "#;
        let config = Config::from_toml(toml).unwrap();

        assert_eq!(
            config.default_providers(),
            vec!["github".to_string(), "work-gitlab".to_string()]
        );
    }

    #[test]
    fn test_default_providers_fallback() {
        let config = Config::from_toml("").unwrap();

        assert_eq!(
            config.default_providers(),
            vec![
                "github".to_string(),
                "gitlab".to_string(),
                "bitbucket".to_string()
            ]
        );
    }

    #[test]
    fn test_custom_provider_uses_default_url_for_type() {
        let toml = r#"
            [providers.my-github]
            type = "github"
            token = "my-token"
        "#;
        let config = Config::from_toml(toml).unwrap();

        let provider = config.resolve_provider("my-github").unwrap();
        // Should use GitHub's default URL since none was specified
        assert_eq!(provider.url, "https://api.github.com");
    }
}
