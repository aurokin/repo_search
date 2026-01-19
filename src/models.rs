use serde::Serialize;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Tabled)]
pub struct Repository {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Owner")]
    pub owner: String,
    #[tabled(rename = "Private")]
    #[tabled(display_with = "display_bool")]
    pub private: bool,
    #[tabled(rename = "Provider")]
    pub provider: String,
    #[tabled(rename = "URL")]
    pub url: String,
    #[tabled(skip)]
    pub full_name: String,
    #[tabled(skip)]
    pub description: Option<String>,
}

fn display_bool(b: &bool) -> String {
    if *b {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResults {
    pub repositories: Vec<Repository>,
    pub total: usize,
}
