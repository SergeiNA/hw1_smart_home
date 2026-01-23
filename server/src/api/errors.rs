use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Room already exists: {0}")]
    RoomAlreadyExists(String),
    #[error("Device already exists: {0}")]
    DeviceAlreadyExists(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::RoomNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::DeviceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::RoomAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
            ApiError::DeviceAlreadyExists(_) => (StatusCode::CONFLICT, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: message,
            code: status.as_u16(),
        });
        (status, body).into_response()
    }
}
