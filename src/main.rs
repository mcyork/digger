use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use dirs::home_dir;
use reqwest::Client;
use tokio;

#[derive(Serialize, Deserialize)]
struct Config {
    api_url: String,
}

impl Config {
    fn load() -> Option<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let config_data = fs::read_to_string(config_path).ok()?;
            serde_json::from_str(&config_data).ok()
        } else {
            None
        }
    }

    fn save(&self) {
        let config_path = Self::config_path();
        let config_data = serde_json::to_string_pretty(self).unwrap();
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(config_path, config_data).unwrap();
    }

    fn config_path() -> PathBuf {
        home_dir().unwrap().join(".digger").join("config.json")
    }
}

#[tokio::main]
async fn main() {
    let matches = App::new("digger")
        .version("1.0")
        .about("A Rust CLI tool to perform DNS lookups using a specified API")
        .arg(Arg::with_name("setup")
            .long("setup")
            .value_name("URL")
            .help("Set the API URL")
            .takes_value(true))
        .arg(Arg::with_name("config")
            .long("config")
            .help("Show the current configuration"))
        .arg(Arg::with_name("DOMAIN")
            .help("The domain to look up")
            .required_unless_one(&["setup", "config"])
            .index(1))
        .arg(Arg::with_name("TYPE")
            .help("The DNS record type")
            .required_unless_one(&["setup", "config"])
            .index(2))
        .get_matches();

    if let Some(url) = matches.value_of("setup") {
        let config = Config { api_url: url.to_string() };
        config.save();
        println!("API URL set to {}", url);
        return;
    }

    if matches.is_present("config") {
        let config = Config::load().expect("Configuration not found. Run 'digger --setup <URL>' first.");
        println!("Current configuration:");
        println!("API URL: {}", config.api_url);
        return;
    }

    let config = Config::load().expect("Configuration not found. Run 'digger --setup <URL>' first.");
    let domain = matches.value_of("DOMAIN").unwrap();
    let record_type = matches.value_of("TYPE").unwrap();

    let client = Client::new();
    let request_body = serde_json::json!({
        "dns_name": domain,
        "dns_type": record_type,
        "dns_servers": ["8.8.8.8", "1.1.1.1"],  // Example servers; you can modify as needed
        "protocol": "UDP"  // Example protocol; you can modify as needed
    });

    let response = client.post(&config.api_url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    let response_text = response.text().await.expect("Failed to read response text");
    println!("Response: {}", response_text);
}
