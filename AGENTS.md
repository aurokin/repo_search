# Agent Guidelines for repo_search

## Architecture

- **Provider trait pattern**: All providers in `src/providers/` implement the `Provider` trait. When adding a new provider type, implement `search()`, `name()`, and `is_authenticated()`.
- **Display names**: Providers accept a `display_name` parameter to show custom provider names (e.g., "work-gitlab") in results instead of generic type names.

## Configuration

- **Backwards compatibility**: Config supports both legacy top-level sections (`[github]`, `[gitlab]`, `[bitbucket]`) and new `[providers.*]` sections. The `migrate_legacy_providers()` function handles this - don't remove it.
- **Provider type inference**: For providers named "github", "gitlab", or "bitbucket", the `type` field is optional. Custom names require an explicit `type`.

## API Behaviors

- **Bitbucket requires auth**: The Bitbucket API requires authentication to search all repositories. Without a token, only `--mine` searches work.
- **Serde ignores unknown fields**: This is intentional for API resilience - don't add `#[serde(deny_unknown_fields)]` to API response structs.

## Async Patterns

- **Concurrent searches**: Uses `tokio::task::JoinSet` for parallel provider searches. Provider instances must be created inside the spawned task due to lifetime requirements.

## Testing

- Run `cargo build --release` to verify changes compile
- Test with `--list-providers` to verify config parsing
- Test searches against real APIs (GitHub/GitLab public APIs work without auth)
