use crate::error::AppErr;
use crate::helpers::AuthResp;
use reqwest::header;
use reqwest::multipart;
use reqwest::Body;
use reqwest::Client;
use reqwest::Error;
use reqwest::Response;
use reqwest::StatusCode;
use std::env;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

pub struct AppData {
    pub temp_counter: Mutex<u64>,
    pub audio_folder: String,
    pub client: Client,
    pub jwt: RwLock<String>,
}

impl AppData {
    pub async fn new() -> Self {
        let app_data = AppData {
            temp_counter: Mutex::new(0),
            audio_folder: env::var("AUDIO_FOLDER").unwrap(),
            client: Client::new(),
            jwt: RwLock::default(),
        };

        app_data.update_jwt().await.unwrap();

        app_data
    }

    pub async fn update_jwt(&self) -> Result<(), AppErr> {
        *self.jwt.write().await = self.get_new_jwt().await?;
        Ok(())
    }

    pub async fn get_counter(&self) -> u64 {
        let mut counter_g = self.temp_counter.lock().await;
        *counter_g += 1;
        *counter_g
    }

    pub async fn get_new_jwt(&self) -> Result<String, AppErr> {
        let mut headers = header::HeaderMap::new();
        headers.insert("accept", "application/json".parse().unwrap());

        let log = env::var("AUTH_LOGIN").unwrap();
        let pass = env::var("AUTH_PASSWORD").unwrap();

        let resp: AuthResp = self
            .client
            .post(format!(
                "http://asrdemo.devmachine.tech/auth/sign-in?username={log}&password={pass}"
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .json()
            .await
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(resp.access_token)
    }

    pub async fn do_file_tt_req(
        &self,
        file_name: &str,
        file_stream: impl Into<Body>,
    ) -> Result<Response, Error> {
        let part = multipart::Part::stream(file_stream).file_name(file_name.to_string());
        let form = multipart::Form::new().part("file", part);

        let mut headers = header::HeaderMap::new();
        headers.insert("accept", "application/json".parse().unwrap());

        let request = self
            .client
            .post("http://asrdemo.devmachine.tech/operation/get_text")
            .bearer_auth(self.jwt.read().await)
            .query(&[("language", "ru")])
            .headers(headers.clone())
            .multipart(form);

        request.send().await
    }
}
