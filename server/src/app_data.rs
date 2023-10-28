use crate::error::AppErr;
use crate::helpers::ext_by_name;
use crate::helpers::ffmpeg_convert;
use crate::helpers::AuthResp;
use crate::helpers::SttResp;
use reqwest::header;
use reqwest::multipart;
use reqwest::Body;
use reqwest::Client;
use reqwest::Error;
use reqwest::Response;
use reqwest::StatusCode;
use std::env;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use youtube_dl::YoutubeDl;

pub struct AppData {
    temp_counter: Mutex<u64>,
    pub temp_folder: String,
    pub client: Client,
    jwt: RwLock<String>,
}

impl AppData {
    pub async fn new() -> Self {
        let app_data = AppData {
            temp_counter: Mutex::new(0),
            temp_folder: env::var("TEMP_FOLDER").unwrap(),
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
        file_stream: impl Into<Body>,
        file_name: &str,
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

    pub async fn file_tt(&self, file: impl Into<Body>, file_name: &str) -> Result<String, AppErr> {
        let result = self.do_file_tt_req(file, file_name).await;

        let response = if let Ok(r) = result {
            r
        } else {
            self.update_jwt().await?;

            return Err(AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "jwt expired",
            ));

            // repeat request
            // app_data
            //     .do_file_tt_req(file_name, file_bytes)
            //     .await
            //     .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        };

        let response: SttResp = response
            .json()
            .await
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(response.ch1.text)
    }

    pub async fn full_cycle(&self, url: &str) -> Result<String, AppErr> {
        let counter = self.get_counter().await;

        let file_name = format!("temp{counter}");

        let path = PathBuf::from(&self.temp_folder);
        let mut ytd = YoutubeDl::new(url);

        ytd.output_template(file_name.clone())
            .extract_audio(true)
            .download_to_async(path)
            .await
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let ext = ext_by_name(&self.temp_folder, &file_name)?;

        let input_file_path = format!("./{}/{file_name}.{ext}", self.temp_folder);
        let output_file_path = format!("./{}/{file_name}.wav", self.temp_folder);

        ffmpeg_convert(&input_file_path, &output_file_path).await?;

        let file_fs = tokio::fs::File::open(output_file_path).await.unwrap();

        self.file_tt(file_fs, &format!("{file_name}.wav")).await
    }
}
