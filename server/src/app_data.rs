use std::env;
use tokio::sync::Mutex;

pub struct AppData {
    pub temp_counter: Mutex<u32>,
    pub audio_folder: String,
    pub model_path: String,
}

impl AppData {
    pub fn new() -> Self {
        AppData {
            temp_counter: Mutex::new(0),
            audio_folder: env::var("AUDIO_FOLDER").unwrap(),
            model_path: env::var("MODEL_PATH").unwrap(),
        }
    }
}
