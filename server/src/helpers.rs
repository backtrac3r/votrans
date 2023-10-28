use crate::app_data::AppData;
use crate::error::AppErr;
use axum::http::StatusCode;
use std::{fs::read_dir, path::PathBuf, result::Result};
use tokio::process::Command;
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

    let path = PathBuf::from(&app_data.temp_folder);
    let mut ytd = YoutubeDl::new(url);

    ytd.output_template(file_name.clone())
        .extract_audio(true)
        .download_to_async(path)
        .await
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ext = ext_by_name(&app_data.temp_folder, &file_name)?;

    let file_name = format!("{file_name}.{ext}");
    let file_path = format!("./{}/{file_name}", app_data.temp_folder);
    let file_fs = tokio::fs::File::open(file_path).await.unwrap();

    app_data.file_tt(file_fs).await
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

pub async fn ffmpeg_convert(
    ffmpeg_input_file_path: &str,
    ffmpeg_output_file_path: &str,
) -> Result<(), AppErr> {
    Command::new("ffmpeg")
        .args(vec![
            "-y",
            "-i",
            &ffmpeg_input_file_path,
            "-ac",
            "1",
            &ffmpeg_output_file_path,
        ])
        .status()
        .await
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("err while spawn ffmpeg task: {e}"),
            )
        })?;

    Command::new("rm")
        .arg(ffmpeg_input_file_path)
        .status()
        .await
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("err while spawn ffmpeg task: {e}"),
            )
        })?;

    Ok(())
}
