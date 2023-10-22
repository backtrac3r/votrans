mod app_data;
mod error;
mod helpers;
mod routes;

use app_data::AppData;
use axum::{routing::post, Router};
use routes::full_cycle;
use std::sync::Arc;
use tokio::fs;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let app_data = Arc::new(AppData::new());

    // clear video & audio dirs
    fs::remove_dir_all(&app_data.audio_folder).await.ok();
    fs::create_dir(&app_data.audio_folder).await.unwrap();

    let app = Router::new()
        .route("/full", post(full_cycle))
        // .route("/ffmpeg", post(ffmpeg_page))
        // .route("/yt", post(yt_dlp))
        // .route("/vosk", post(vosk_page))
        .with_state(app_data);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
