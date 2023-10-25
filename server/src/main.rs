mod app_data;
mod error;
mod helpers;
mod routes;

use app_data::AppData;
use axum::{routing::post, Router};
use routes::full_cycle_handler;
use std::{env, sync::Arc};
use tokio::fs;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let app_data = Arc::new(AppData::new().await);

    // clear video & audio dirs
    fs::remove_dir_all(&app_data.audio_folder).await.ok();
    fs::create_dir(&app_data.audio_folder).await.unwrap();

    let app = Router::new()
        .route("/full", post(full_cycle_handler))
        // .route("/ffmpeg", post(ffmpeg_page))
        // .route("/yt", post(yt_dlp))
        // .route("/vosk", post(vosk_page))
        .with_state(app_data);

    let addr = &format!("0.0.0.0:{}", env::var("SERVER_PORT").unwrap())
        .parse()
        .unwrap();

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
