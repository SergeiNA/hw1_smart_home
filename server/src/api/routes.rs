use super::handlers::{device, report, room};
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn spawn_router(state: AppState) -> Router {
    Router::new()
        // Room routes
        .route("/api/v1/room/room_list", get(room::room_list))
        .route("/api/v1/room/add_room", post(room::add_room))
        .route("/api/v1/room/{room_name}", delete(room::delete_room))
        .route("/api/v1/room/{room_name}", get(room::get_room))
        // Device routes
        .route("/api/v1/room/{room_name}/devices", get(device::device_list))
        .route("/api/v1/room/{room_name}/device", post(device::add_device))
        .route(
            "/api/v1/room/{room_name}/device/{device_name}",
            delete(device::delete_device),
        )
        // Reports routes
        .route("/api/v1/home/report", get(report::home_report))
        .route("/api/v1/room/{room_name}/report", get(report::room_report))
        .route(
            "/api/v1/room/{room_name}/device/{device_name}/report",
            get(report::device_report),
        )
        .with_state(state)
}
