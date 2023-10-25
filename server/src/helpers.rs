use crate::app_data::AppData;
use crate::error::AppErr;
use axum::http::StatusCode;
use reqwest::header;
use reqwest::multipart;
use reqwest::Client;
use std::env;
use std::{fs::read_dir, path::PathBuf, result::Result};
use youtube_dl::YoutubeDl;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResp {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "token_type")]
    pub token_type: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SttResp {
    #[serde(rename = "0")]
    pub ch1: N0,
    #[serde(rename = "1")]
    pub ch2: N1,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N0 {
    #[serde(rename = "file_name")]
    pub file_name: String,
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N1 {
    #[serde(rename = "file_name")]
    pub file_name: String,
    pub text: String,
}

pub async fn full_cycle(counter: u64, url: &str, data: &AppData) -> Result<String, AppErr> {
    let output_name = format!("temp{counter}");

    let path = PathBuf::from(&data.audio_folder);
    let mut ytd = YoutubeDl::new(url);

    ytd.output_template(output_name.clone())
        .extract_audio(true)
        .download_to_async(path)
        .await
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ext = ext_by_name(&data.audio_folder, &output_name)?;

    let file_name = format!("{output_name}.{ext}");
    let file_path = format!("./{}/{file_name}", data.audio_folder);

    file_to_txt(&file_name, &file_path, data).await
}

#[allow(clippy::cast_precision_loss)]
pub async fn file_to_txt(
    file_name: &str,
    file_path: &str,
    data: &AppData,
) -> Result<String, AppErr> {
    let file_fs = tokio::fs::File::open(file_path).await.unwrap();
    let part = multipart::Part::stream(file_fs)
        .file_name(file_name.to_string())
        .mime_str("audio/x-m4a")
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let form = multipart::Form::new().part("file", part);

    let mut headers = header::HeaderMap::new();
    headers.insert("accept", "application/json".parse().unwrap());

    let request = data
        .client
        .post("http://asrdemo.devmachine.tech/operation/get_text")
        .bearer_auth(data.jwt.read().await)
        .query(&[("language", "ru")])
        .headers(headers.clone())
        .multipart(form);

    let result = request.send().await;
    let response = if let Ok(r) = result {
        r
    } else {
        data.update_jwt().await?;

        let file_fs = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let part = multipart::Part::stream(file_fs)
            .file_name(file_name.to_string())
            .mime_str("audio/x-m4a")
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let form = multipart::Form::new().part("file", part);

        let request = data
            .client
            .post("http://asrdemo.devmachine.tech/operation/get_text")
            .bearer_auth(data.jwt.read().await)
            .query(&[("language", "ru")])
            .headers(headers)
            .multipart(form);

        request
            .bearer_auth(data.jwt.read().await)
            .send()
            .await
            .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let response: SttResp = response
        .json()
        .await
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(response.ch1.text)
}

pub fn ext_by_name(path: &str, file_name: &str) -> Result<String, AppErr> {
    let dir = read_dir(path).map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("error while read dir: {e}"),
        )
    })?;

    let dir = dir.filter_map(Result::ok);

    for file in dir {
        let p = file.path().to_string_lossy().into_owned();
        if p.contains(file_name) {
            return Ok(file
                .path()
                .extension()
                .ok_or_else(|| {
                    AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "error while read ext")
                })?
                .to_string_lossy()
                .into_owned());
        }
    }

    Err(AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "no file"))
}

pub async fn get_new_jwt() -> Result<String, AppErr> {
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", "application/json".parse().unwrap());

    let log = env::var("AUTH_LOGIN").unwrap();
    let pass = env::var("AUTH_PASSWORD").unwrap();

    let client = Client::new();
    let resp: AuthResp = client
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
