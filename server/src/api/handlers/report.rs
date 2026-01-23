use crate::{
    api::errors::ApiError,
    models::response::{DeviceReportResponse, HomeReportResponse, RoomReportResponse},
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
};
use smart_home::smart_home::DeviceAccessError;
use smart_home::traits::Information;

pub async fn home_report(
    State(state): State<AppState>,
) -> Result<Json<HomeReportResponse>, ApiError> {
    let home = state.read().await;
    let report = home.info();
    Ok(Json(HomeReportResponse {
        name: home.name(),
        report,
    }))
}

pub async fn room_report(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<RoomReportResponse>, ApiError> {
    let home = state.read().await;
    if let Some(room) = home.view_room(&room_name) {
        Ok(Json(RoomReportResponse {
            name: room_name.clone(),
            report: room.info(),
        }))
    } else {
        Err(ApiError::RoomNotFound(room_name.clone()))
    }
}

pub async fn device_report(
    State(state): State<AppState>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<Json<DeviceReportResponse>, ApiError> {
    let home = state.write().await;
    home.device(&room_name, &device_name)
        .map(|device| {
            Json(DeviceReportResponse {
                name: device_name.clone(),
                report: device.info(),
            })
        })
        .map_err(|e| match e {
            DeviceAccessError::DeviceAccess(e) => ApiError::DeviceNotFound(e.to_string()),
            DeviceAccessError::RoomAccess(e) => ApiError::RoomNotFound(e.to_string()),
        })
}
