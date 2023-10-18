mod app_data;
mod error;
mod helpers;
mod routes;

use app_data::AppData;
use axum::{routing::post, Router};
use routes::{ffmpeg_page, full_cycle, vosk_page, yt_dlp};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ffmpeg", post(ffmpeg_page))
        .route("/yt", post(yt_dlp))
        .route("/vosk", post(vosk_page))
        .route("/full", post(full_cycle))
        .with_state(AppData::new());

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
