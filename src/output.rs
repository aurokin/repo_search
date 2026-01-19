use tabled::{settings::Style, Table};

use crate::models::{Repository, SearchResults};

pub fn print_results(repos: Vec<Repository>, as_json: bool) {
    if repos.is_empty() {
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&SearchResults {
                    repositories: vec![],
                    total: 0,
                })
                .unwrap()
            );
        } else {
            println!("No repositories found.");
        }
        return;
    }

    if as_json {
        let results = SearchResults {
            total: repos.len(),
            repositories: repos,
        };
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        println!("Found {} repositories:\n", repos.len());
        let table = Table::new(&repos).with(Style::rounded()).to_string();
        println!("{}", table);
    }
}
