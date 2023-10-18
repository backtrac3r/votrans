use axum::{http::StatusCode, response::IntoResponse, Json};

pub struct AppErr {
    pub code: StatusCode,
    pub msg: String,
}

impl AppErr {
    pub fn new(code: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
        }
    }
}

impl IntoResponse for AppErr {
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(ResponseErr { msg: self.msg })).into_response()
    }
}

#[derive(serde::Serialize)]
pub struct ResponseErr {
    msg: String,
}
