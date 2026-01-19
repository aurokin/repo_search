# repo_search

A fast CLI tool for searching git repositories across GitHub, GitLab, and Bitbucket.

## Features

- Search repositories by name and description across multiple providers
- Support for GitHub, GitLab, and Bitbucket (including self-hosted instances)
- Define custom provider instances (e.g., work GitLab, personal Bitbucket)
- Filter to only your own repositories with `--mine`
- Filter to a specific owner with `--owner`
- Show private repositories when authenticated
- Table or JSON output formats
- Concurrent searches across all providers

## Installation

### From Source

```bash
git clone https://github.com/yourusername/repo_search.git
cd repo_search
cargo build --release
```

The binary will be at `./target/release/repo_search`.

## Usage

```bash
# Search all default providers
repo_search "rust cli"

# Search specific provider(s)
repo_search -p github "rust cli"
repo_search -p github -p gitlab "rust cli"

# Search a custom provider by name
repo_search -p work-gitlab "internal project"

# Search all configured providers
repo_search -p all "query"

# Only show your own repositories
repo_search --mine "my project"

# Filter by owner
repo_search --owner rust-lang "rust"

# Limit results per provider
repo_search -l 5 "query"

# Output as JSON
repo_search --json "query"

# List all configured providers
repo_search --list-providers
```

### Options

```
Usage: repo_search [OPTIONS] [QUERY]

Arguments:
  [QUERY]  Search query (required unless using --list-providers)

Options:
  -p, --provider <PROVIDER>  Provider(s) to search (can specify multiple)
  -u, --url <URL>            Custom instance URL (overrides provider URL)
  -m, --mine                 Only show repositories you own
      --owner <OWNER>        Only show repositories owned by this user/org
  -l, --limit <LIMIT>        Maximum results per provider
      --json                 Output as JSON
      --list-providers       List all configured providers and exit
  -h, --help                 Print help
  -V, --version              Print version
```

## Configuration

Configuration file location: `~/.config/repo_search/config.toml`

### Basic Configuration

```toml
[defaults]
providers = ["github", "gitlab"]  # Providers to search by default
limit = 10                         # Default results per provider

[providers.github]
token = "ghp_xxxxxxxxxxxx"

[providers.gitlab]
token = "glpat-xxxxxxxxxxxx"

[providers.bitbucket]
token = "your-app-password"
```

### Custom Provider Instances

You can define multiple instances of the same provider type with different URLs and credentials:

```toml
[defaults]
providers = ["github", "work-gitlab"]
limit = 10

# Built-in providers (type is inferred from name)
[providers.github]
token = "ghp_xxxxxxxxxxxx"

[providers.gitlab]
token = "glpat-xxxxxxxxxxxx"

# Custom providers (type is required)
[providers.work-gitlab]
type = "gitlab"
url = "https://gitlab.mycompany.com"
token = "work-gitlab-token"

[providers.work-bitbucket]
type = "bitbucket"
url = "https://bitbucket.mycompany.com"
token = "work-bitbucket-token"

[providers.personal-gitlab]
type = "gitlab"
url = "https://gitlab.personal.io"
token = "personal-token"
```

### Environment Variables

Environment variables override config file values for built-in providers:

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | GitHub personal access token |
| `GITHUB_URL` | GitHub API URL (for GitHub Enterprise) |
| `GITLAB_TOKEN` | GitLab personal access token |
| `GITLAB_URL` | GitLab instance URL |
| `BITBUCKET_TOKEN` | Bitbucket app password |
| `BITBUCKET_URL` | Bitbucket API URL |

## Authentication

### GitHub

Create a personal access token at https://github.com/settings/tokens with `repo` scope for private repository access.

### GitLab

Create a personal access token at https://gitlab.com/-/profile/personal_access_tokens with `read_api` scope.

### Bitbucket

Create an app password at https://bitbucket.org/account/settings/app-passwords/ with `Repositories: Read` permission.

**Note:** Bitbucket requires authentication to search repositories. Without a token, Bitbucket searches will fail unless using `--mine`.

## Output Formats

### Table (default)

```
Found 3 repositories:

╭──────────┬───────────────┬─────────┬──────────┬───────────────────────────────────────╮
│ Name     │ Owner         │ Private │ Provider │ URL                                   │
├──────────┼───────────────┼─────────┼──────────┼───────────────────────────────────────┤
│ rust     │ rust-lang     │ No      │ github   │ https://github.com/rust-lang/rust     │
│ Rust     │ TheAlgorithms │ No      │ github   │ https://github.com/TheAlgorithms/Rust │
│ rustdesk │ rustdesk      │ No      │ github   │ https://github.com/rustdesk/rustdesk  │
╰──────────┴───────────────┴─────────┴──────────┴───────────────────────────────────────╯
```

### JSON

```bash
repo_search --json "rust"
```

```json
{
  "repositories": [
    {
      "name": "rust",
      "owner": "rust-lang",
      "private": false,
      "provider": "github",
      "url": "https://github.com/rust-lang/rust",
      "full_name": "rust-lang/rust",
      "description": "Empowering everyone to build reliable and efficient software."
    }
  ],
  "total": 1
}
```

## Examples

### Search across work and personal GitLab instances

```toml
# ~/.config/repo_search/config.toml
[defaults]
providers = ["work-gitlab", "personal-gitlab"]

[providers.work-gitlab]
type = "gitlab"
url = "https://gitlab.company.com"
token = "work-token"

[providers.personal-gitlab]
type = "gitlab"
url = "https://gitlab.com"
token = "personal-token"
```

```bash
# Search both instances
repo_search "my project"

# Search only work GitLab
repo_search -p work-gitlab "internal"
```

### Quick search on GitHub only

```bash
GITHUB_TOKEN=ghp_xxx repo_search -p github "awesome project"
```

## Development

### Commands

| Task | Command |
|------|---------|
| Build (debug) | `cargo build` |
| Build (release) | `cargo build --release` |
| Run (dev mode) | `cargo run -- "query"` |
| Formatting | `cargo fmt` |
| Linting | `cargo clippy` |
| Tests | `cargo test` |

### Development Workflow

```bash
# Format code
cargo fmt

# Lint and catch common issues
cargo clippy

# Build and run in development
cargo run -- --list-providers
cargo run -- -p github "rust" -l 5

# Run tests
cargo test

# Build optimized release binary
cargo build --release
```

## License

MIT
