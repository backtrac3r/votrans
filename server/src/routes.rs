use crate::{app_data::AppData, error::AppErr, helpers::full_cycle};
use api::Ytdlp;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn full_cycle_handler(
    State(data): State<Arc<AppData>>,
    Json(req): Json<Ytdlp>,
) -> Result<String, AppErr> {
    let mut counter_g = data.temp_counter.lock().await;
    *counter_g += 1;
    let counter = *counter_g;
    drop(counter_g);

    full_cycle(counter, &req.url, &data).await
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
