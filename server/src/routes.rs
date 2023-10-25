use crate::{
    app_data::AppData,
    error::AppErr,
    helpers::{file_tt, full_cycle},
};
use api::Ytdlp;
use axum::{
    extract::{Multipart, State},
    Json,
};
use std::sync::Arc;

pub async fn url_tt_handler(
    State(app_data): State<Arc<AppData>>,
    Json(req): Json<Ytdlp>,
) -> Result<String, AppErr> {
    println!("new url_tt req");
    let counter = app_data.get_counter().await;

    full_cycle(counter, &req.url, &app_data).await
}

pub async fn file_tt_handler(
    State(app_data): State<Arc<AppData>>,
    mut multipart: Multipart,
) -> Result<String, AppErr> {
    println!("new file_tt req");
    while let Ok(Some(field)) = multipart.next_field().await {
        let Ok(file_bytes) = field.bytes().await else {
            continue;
        };

        let counter = app_data.get_counter().await;

        let file_name = format!("temp{counter}");

        return file_tt(&file_name, file_bytes, &app_data).await;
    }

    Ok(String::new())
}
