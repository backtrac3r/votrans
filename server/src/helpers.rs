use crate::error::AppErr;
use axum::http::StatusCode;
use std::{fs::read_dir, result::Result};
use tokio::process::Command;

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
    pub ch2: Option<N1>,
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
            "-loglevel",
            "quiet",
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
