use std::env;

use reqwest::Client;

pub struct Config {
    pub client: Client,
    pub server_port: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            server_port: env::var("SERVER_PORT").unwrap(),
        }
    }
}
