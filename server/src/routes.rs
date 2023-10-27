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
use tokio::io::AsyncWriteExt;

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
        let Some(file_name) = field.file_name() else {
            continue;
        };
        dbg!(&file_name);
        let file_name = file_name.to_string();

        let Ok(file_bytes) = field.bytes().await else {
            continue;
        };

        tokio::fs::create_dir(format!("{file_name}.m4a"))
            .await
            .unwrap();
        let mut file = tokio::fs::File::open(format!("{file_name}.m4a"))
            .await
            .unwrap();
        file.write_all(&file_bytes).await.unwrap();

        return file_tt(&file_name, file_bytes, &app_data).await;
    }

    Ok(String::new())
}
