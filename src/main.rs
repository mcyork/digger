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

#[derive(Deserialize)]
struct DnsResult {
    dns_name: String,
    dns_type: String,
    results: Vec<ServerResult>,
}

#[derive(Deserialize)]
struct ServerResult {
    friendly_name: String,
    server: String,
    results: Vec<RecordResult>,
}

#[derive(Deserialize)]
struct RecordResult {
    query_name: String,
    record: String,
    r#type: String,
    ttl: u32,
    authoritative: bool,
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
        .arg(Arg::with_name("advanced")
            .long("advanced")
            .help("Enable advanced output"))
        .arg(Arg::with_name("dns_servers")
            .long("dns-servers")
            .value_name("DNS_SERVERS")
            .help("Comma-separated list of additional DNS servers to query")
            .takes_value(true))
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
    let advanced = matches.is_present("advanced");

    let default_dns_servers = vec!["8.8.8.8", "1.1.1.1"];
    let dns_servers: Vec<String> = matches.value_of("dns_servers")
        .map(|s| s.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_else(|| default_dns_servers.iter().map(|&s| s.to_string()).collect());

    let client = Client::new();
    let request_body = serde_json::json!({
        "dns_name": domain,
        "dns_type": record_type,
        "dns_servers": dns_servers,
        "protocol": "UDP",
        "advanced": advanced
    });

    let response = client.post(&config.api_url)
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    let response_text = response.text().await.expect("Failed to read response text");

    let dns_result: DnsResult = serde_json::from_str(&response_text).expect("Failed to parse response JSON");

    println!("DNS Lookup Results:");
    println!("DNS Name: {}", dns_result.dns_name);
    println!("DNS Type: {}", dns_result.dns_type);
    for server_result in dns_result.results {
        println!("\nResults from DNS Server: {} ({})", server_result.friendly_name, server_result.server);
        for answer in server_result.results {
            if advanced {
                println!(
                    "{}\t{}\tIN\t{}\t{}",
                    answer.query_name, answer.ttl, answer.r#type, answer.record
                );
            } else {
                println!("- {}", answer.record);
            }
        }
    }
}
