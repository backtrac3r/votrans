use crate::{
    app_data::AppData,
    error::AppErr,
    helpers::{convert_to_wav, file_ext_from_url, vosk_wav, Ytdlp},
};
use axum::{extract::State, http::StatusCode, Json};
use std::{path::PathBuf, sync::Arc};
use youtube_dl::YoutubeDl;

pub async fn full_cycle(
    State(data): State<Arc<AppData>>,
    Json(req): Json<Ytdlp>,
) -> Result<String, AppErr> {
    let output_name = format!("temp{}", data.temp_counter.lock().await);
    *data.temp_counter.lock().await += 1;

    let path = PathBuf::from(&data.audio_folder);
    let mut ytd = YoutubeDl::new(&req.url);

    ytd.output_template(output_name.clone())
        .extract_audio(true)
        .download_to_async(path)
        .await
        .map_err(|e| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ext = file_ext_from_url(&req.url)?;

    let ffmpeg_input_file_path = format!("./{}/{output_name}.{ext}", data.audio_folder);
    let ffmpeg_output_file_path = format!("./{}/{output_name}.wav", data.audio_folder);

    convert_to_wav(&ffmpeg_input_file_path, &ffmpeg_output_file_path).await?;

    vosk_wav(ffmpeg_output_file_path, &data.model_path)
}

// pub async fn ffmpeg_page(Json(path): Json<Ytdlp>) -> Result<String, AppErr> {
//     let input_path = format!("./downloads/{}", path.url);

//     let name_without_ext = path
//         .url
//         .split('.')
//         .next()
//         .ok_or_else(|| AppErr::new(StatusCode::INTERNAL_SERVER_ERROR, "invalid url"))?;

//     let output_name = format!("./ffmpeg/{name_without_ext}.wav");

//     let mut ffmpeg = process::Command::new("ffmpeg")
//         .args(vec![
//             "-y",
//             "-i",
//             &input_path,
//             "-map",
//             "0:a",
//             "-ac",
//             "1",
//             &output_name,
//         ])
//         .spawn()
//         .map_err(|e| {
//             AppErr::new(
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("err while spawn ffmpeg task: {e}"),
//             )
//         })?;

//     ffmpeg.wait().map_err(|e| {
//         AppErr::new(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("ffmpeg err: {e}"),
//         )
//     })?;

//     Ok(format!("{output_name}.wav"))
// }

// pub async fn yt_dlp(
//     State(data): State<Arc<AppData>>,
//     Json(url): Json<Ytdlp>,
// ) -> Result<String, AppErr> {
//     let output_name = format!("temp{}", data.temp_counter.lock().await);
//     *data.temp_counter.lock().await += 1;

//     let path = PathBuf::from("./downloads");
//     let ytd = YoutubeDl::new(&url.url);

//     ytd.download_to(path).map_err(|e| {
//         AppErr::new(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Could not download video: {e}"),
//         )
//     })?;

//     Ok(format!("{output_name}.webm"))
// }

// pub async fn vosk_page(
//     State(data): State<Arc<AppData>>,
//     url: Json<Ytdlp>,
// ) -> Result<String, AppErr> {
//     vosk_wav(url.url.clone(), &data.model_path)
// }
