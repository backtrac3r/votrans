mod app_data;
mod error;
mod helpers;
mod routes;

use crate::routes::file_tt_handler;
use app_data::AppData;
use axum::{extract::DefaultBodyLimit, routing::post, Router};
use routes::url_tt_handler;
use std::{env, sync::Arc};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let app_data = Arc::new(AppData::new().await);

    // clear video & audio dirs
    tokio::fs::remove_dir_all(&app_data.audio_folder).await.ok();
    tokio::fs::create_dir(&app_data.audio_folder).await.unwrap();

    let app = Router::new()
        .route("/url_tt", post(url_tt_handler))
        .route("/file_tt", post(file_tt_handler))
        .layer(DefaultBodyLimit::max(100_000_000))
        .with_state(app_data);

    let addr = &format!("0.0.0.0:{}", env::var("SERVER_PORT").unwrap())
        .parse()
        .unwrap();

    println!("server online");

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
