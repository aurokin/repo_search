use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "repo_search")]
#[command(
    version,
    about = "Search git repositories across GitHub, GitLab, and Bitbucket"
)]
pub struct Args {
    /// Search query (required unless using --list-providers)
    pub query: Option<String>,

    /// Provider(s) to search (can specify multiple: -p github -p work-gitlab)
    /// Use "all" to search all configured providers
    #[arg(short, long)]
    pub provider: Vec<String>,

    /// Custom instance URL (overrides the URL for specified providers)
    #[arg(short = 'u', long)]
    pub url: Option<String>,

    /// Only show repositories you own
    #[arg(short, long)]
    pub mine: bool,

    /// Maximum results per provider (default: 10, or from config)
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// List all configured providers and exit
    #[arg(long)]
    pub list_providers: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
