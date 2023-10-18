use tokio::sync::Mutex;

pub struct AppData {
    pub temp_counter: Mutex<u32>,
}

impl AppData {
    pub fn new() -> Self {
        AppData {
            temp_counter: Mutex::new(0),
        }
    }
}
