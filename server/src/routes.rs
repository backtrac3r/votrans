use crate::{app_data::AppData, error::AppErr, helpers::ffmpeg_convert};
use api::Ytdlp;
use axum::{
    extract::{Multipart, State},
    Json,
};
use std::sync::Arc;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
};

pub async fn url_tt_handler(
    State(app_data): State<Arc<AppData>>,
    Json(req): Json<Ytdlp>,
) -> Result<String, AppErr> {
    println!("new url_tt req");

    app_data.full_cycle(&req.url).await
}

pub async fn file_tt_handler(
    State(app_data): State<Arc<AppData>>,
    mut multipart: Multipart,
) -> Result<String, AppErr> {
    println!("new file_tt req");
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name().unwrap() == "file" {
            let file_name = format!("temp{}", app_data.get_counter().await.to_string());

            let Ok(file_bytes) = field.bytes().await else {
                continue;
            };

            let input_file_path = format!("./{}/{file_name}", app_data.temp_folder);

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&input_file_path)
                .await
                .unwrap();

            file.write_all(&file_bytes).await.unwrap();

            let output_file_path = format!("./{}/{file_name}.wav", app_data.temp_folder);

            ffmpeg_convert(&input_file_path, &output_file_path).await?;

            let file = File::open(&output_file_path).await.unwrap();

            return app_data.file_tt(file, &format!("{file_name}.wav")).await;
        }
    }

    Ok(String::new())
}
