use crate::models::response::MessageResponse;
use crate::{
    api::errors::ApiError,
    models::{
        request::AddRoomRequest,
        response::{RoomListResponse, RoomResponse},
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
};
use smart_home::smart_room::SmartRoom;
use smart_home::traits::Information;

pub async fn room_list(State(state): State<AppState>) -> Result<Json<RoomListResponse>, ApiError> {
    let home = state.read().await;
    let rooms = home.rooms().iter().map(|r| r.name().to_string()).collect();
    Ok(Json(RoomListResponse { rooms }))
}

pub async fn add_room(
    State(state): State<AppState>,
    Json(payload): Json<AddRoomRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;
    if payload.name.is_empty() {

    }
    if home.view_room(&payload.name).is_some() {
        return Err(ApiError::RoomAlreadyExists(payload.name.clone()));
    }

    home.add_room(SmartRoom::new(payload.name.clone()));

    Ok(Json(MessageResponse {
        message: format!("Room '{}' added successfully", payload.name),
    }))
}

pub async fn delete_room(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;
    if home.view_room(&room_name).is_none() {
        return Err(ApiError::RoomNotFound(room_name.clone()));
    }

    home.remove_room(&room_name);

    Ok(Json(MessageResponse {
        message: format!("Room '{}' deleted successfully", room_name),
    }))
}

pub async fn get_room(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<RoomResponse>, ApiError> {
    let home = state.read().await;
    if let Some(room) = home.view_room(&room_name) {
        Ok(Json(RoomResponse {
            name: room_name.clone(),
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
