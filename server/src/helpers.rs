use crate::app_data::AppData;
use crate::error::AppErr;
use axum::http::StatusCode;
use reqwest::Body;
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
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N1 {
    #[serde(rename = "file_name")]
    pub file_name: String,
    #[serde(rename = "text")]
    pub text: String,
}

pub async fn full_cycle(counter: u64, url: &str, app_data: &AppData) -> Result<String, AppErr> {
    let file_name = format!("temp{counter}");

    let path = PathBuf::from(&app_data.audio_folder);
    let mut ytd = YoutubeDl::new(url);

    ytd.output_template(file_name.clone())
        .extract_audio(true)
        .download_to_async(path)
        .await
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ext = ext_by_name(&app_data.audio_folder, &file_name)?;

    let file_name = format!("{file_name}.{ext}");
    let file_path = format!("./{}/{file_name}", app_data.audio_folder);
    let file_fs = tokio::fs::File::open(file_path).await.unwrap();

    file_tt(&file_name, file_fs, app_data).await
}

pub async fn file_tt(
    file_name: &str,
    file_bytes: impl Into<Body>,
    app_data: &AppData,
) -> Result<String, AppErr> {
    let result = app_data.do_file_tt_req(file_name, file_bytes).await;
    dbg!();

    let response = if let Ok(r) = result {
        dbg!();
        r
    } else {
        app_data.update_jwt().await?;
        dbg!();
        return Err(AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "jwt expired",
        ));

        // app_data
        //     .do_file_tt_req(file_name, file_bytes)
        //     .await
        //     .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    dbg!();
    let response: SttResp = response
        .json()
        .await
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    dbg!();

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
