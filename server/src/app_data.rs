use crate::error::AppErr;
use crate::helpers::get_new_jwt;
use reqwest::blocking;
use reqwest::Client;
use std::env;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

pub struct AppData {
    pub temp_counter: Mutex<u64>,
    pub audio_folder: String,
    pub blocking_client: blocking::Client,
    pub client: Client,
    pub jwt: RwLock<String>,
}

impl AppData {
    pub async fn new() -> Self {
        AppData {
            temp_counter: Mutex::new(0),
            audio_folder: env::var("AUDIO_FOLDER").unwrap(),
            client: Client::new(),
            blocking_client: blocking::Client::new(),
            jwt: RwLock::new(get_new_jwt().await.unwrap()),
        }
    }

    pub async fn update_jwt(&self) -> Result<(), AppErr> {
        *self.jwt.write().await = get_new_jwt().await?;
        Ok(())
    }
}
