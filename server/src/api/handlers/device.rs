use crate::models::request::{AddDeviceRequest, DeviceType};
use crate::models::response::{MessageResponse};
use crate::{api::errors::ApiError, models::response::DeviceListResponse, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
};
use smart_home::smart_devices::{Celsius, Device, Watt};
use smart_home::traits::Information;

pub async fn device_list(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<DeviceListResponse>, ApiError> {
    let home = state.read().await;
    if let Some(room) = home.view_room(&room_name) {
        Ok(Json(DeviceListResponse {
            devices: room
                .devices()
                .iter()
                .map(|d| d.name().to_string())
                .collect(),
        }))
    } else {
        Err(ApiError::RoomNotFound(room_name.clone()))
    }
}

pub async fn add_device(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
    Json(payload): Json<AddDeviceRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;
    if let Some(room) = home.get_room(&room_name) {
        if room.view_device(&payload.name).is_some() {
            return Err(ApiError::DeviceAlreadyExists(payload.name.clone()));
        }
        let device = match payload.device_type {
            DeviceType::Outlet { power } => Device::new_outlet(
                payload.name.clone(),
                smart_home::smart_devices::OutletState::On,
                power as Watt,
            ),
            DeviceType::Thermometer { temperature } => {
                Device::new_thermometer(payload.name.clone(), temperature as Celsius)
            }
        };
        room.add_device(payload.name.clone(), device);
        Ok(Json(MessageResponse {
            message: format!(
                "Device '{}' added successfully to room '{}'",
                payload.name, room_name
            ),
        }))
    } else {
        Err(ApiError::RoomNotFound(room_name.clone()))
    }
}

pub async fn delete_device(
    State(state): State<AppState>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;
    if let Some(room) = home.get_room(&room_name) {
        if room.view_device(&device_name).is_none() {
            return Err(ApiError::DeviceNotFound(device_name));
        }
        room.remove_device(&device_name);
        Ok(Json(MessageResponse {
            message: format!("Device '{}' deleted from room '{}'", device_name, room_name),
        }))
    } else {
        Err(ApiError::RoomNotFound(room_name))
    }
}
