use crate::{
    app_data::AppData,
    error::AppErr,
    helpers::{vosk_wav, Ytdlp},
};
use axum::{extract::State, http::StatusCode, Json};
use std::process;
use std::{path::PathBuf, sync::Arc};
use youtube_dl::YoutubeDl;

pub async fn ffmpeg_page(path: Json<Ytdlp>) -> Result<String, AppErr> {
    // println!("got ffmpeg");
    // println!("{:?}", std::env::current_dir().unwrap().to_str().unwrap());

    let input_path = format!("./downloads/{}", path.url);

    let name_without_ext = path
        .url
        .split('.')
        .next()
        .ok_or_else(|| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "invalid url"))?;

    let output_name = format!("./ffmpeg/{name_without_ext}.wav");

    let mut ffmpeg = process::Command::new("ffmpeg")
        .args(vec![
            "-y",
            "-i",
            &input_path,
            "-map",
            "0:a",
            "-ac",
            "1",
            &output_name,
        ])
        .spawn()
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("err while spawn ffmpeg task: {e}"),
            )
        })?;

    ffmpeg.wait().map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("ffmpeg err: {e}"),
        )
    })?;

    println!("done ffmpeg");

    // println!("{}", String::from_utf8(ffmpeg.stdout).unwrap());

    Ok(format!("{output_name}.wav"))
}

pub async fn yt_dlp(State(data): State<Arc<AppData>>, url: Json<Ytdlp>) -> Result<String, AppErr> {
    println!("got reqwest {}", url.url);
    let output_name = format!("temp{}", data.temp_counter.lock().await);
    *data.temp_counter.lock().await += 1;

    // let link = "https://www.youtube.com/watch?v=uTO0KnDsVH0";
    let path = PathBuf::from("./downloads");
    let ytd = YoutubeDl::new(&url.url);

    // start download
    ytd.download_to(path).map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not download video: {e}"),
        )
    })?;

    // print out the download path
    println!("Your download filename: {output_name}");

    Ok(format!("{output_name}.webm"))
}

pub async fn vosk_page(url: Json<Ytdlp>) -> Result<String, AppErr> {
    vosk_wav(url.url.clone())
}

pub async fn full_cycle(
    State(data): State<Arc<AppData>>,
    url: Json<Ytdlp>,
) -> Result<String, AppErr> {
    // YT_DLP
    println!("got reqwest {}", url.url);
    let output_name = format!("temp{}", data.temp_counter.lock().await);
    *data.temp_counter.lock().await += 1;
    dbg!();

    dbg!();
    // let link = "https://www.youtube.com/watch?v=uTO0KnDsVH0";
    dbg!();
    let path = PathBuf::from("./downloads");
    dbg!();
    let ytd = YoutubeDl::new(&url.url);
    dbg!();

    // start download
    ytd.download_to(path)
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    dbg!();

    // print out the download path
    println!("Your download filename: {output_name}");

    // FFMPEG
    // println!("got ffmpeg");
    // println!("{:?}", std::env::current_dir().unwrap().to_str().unwrap());
    let input_path = format!("./downloads/{output_name}.webm");

    dbg!();
    let name_without_ext = output_name
        .split('.')
        .next()
        .ok_or_else(|| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "invalid url"))?;

    dbg!();
    let output_path_and_name = format!("./ffmpeg/{name_without_ext}.wav");

    dbg!();
    let mut ffmpeg = process::Command::new("ffmpeg")
        .args(vec![
            "-y",
            "-i",
            &input_path,
            "-map",
            "0:a",
            "-ac",
            "1",
            &output_path_and_name,
        ])
        .spawn()
        .map_err(|e| {
            AppErr::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("err while spawn ffmpeg task: {e}"),
            )
        })?;
    dbg!();

    ffmpeg.wait().map_err(|e| {
        AppErr::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("ffmpeg err: {e}"),
        )
    })?;

    dbg!();
    println!("done ffmpeg");
    println!("{output_path_and_name}");

    // VOSK
    dbg!();
    vosk_wav(output_path_and_name)
}
