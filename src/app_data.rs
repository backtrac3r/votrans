use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppData {
    pub temp_counter: Arc<Mutex<u32>>,
}

impl AppData {
    pub fn new() -> Self {
        AppData {
            temp_counter: Arc::new(Mutex::new(0)),
        }
    }
}
