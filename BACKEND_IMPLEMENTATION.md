# Backend Implementation Guide

## Overview

This guide describes how to implement a REST API backend for the Smart Home service using **Axum** framework.

---

## Dependencies

Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"

[dev-dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio-test = "0.4"
```

---

## Project Structure

```
src/
├── main.rs              # Entry point, server setup
├── lib.rs               # Library exports
├── api/
│   ├── mod.rs           # API module
│   ├── routes.rs        # Route definitions
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── rooms.rs     # Room handlers
│   │   ├── devices.rs   # Device handlers
│   │   └── reports.rs   # Report handlers
│   └── error.rs         # API error types
├── models/
│   ├── mod.rs
│   ├── requests.rs      # Request DTOs
│   └── responses.rs     # Response DTOs
└── state.rs             # Application state
```

---

## REST API Design

### Base URL
```
http://localhost:3000/api/v1
```

### Endpoints

#### Rooms

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/rooms` | List all rooms |
| `GET` | `/rooms/{room_name}` | Get room details |
| `POST` | `/rooms` | Create a new room |
| `DELETE` | `/rooms/{room_name}` | Delete a room |

#### Devices

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/rooms/{room_name}/devices` | List devices in a room |
| `GET` | `/rooms/{room_name}/devices/{device_name}` | Get device details |
| `POST` | `/rooms/{room_name}/devices` | Add device to a room |
| `DELETE` | `/rooms/{room_name}/devices/{device_name}` | Remove device from a room |

#### Reports

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/report` | Get full home report |
| `GET` | `/rooms/{room_name}/report` | Get room report |
| `GET` | `/rooms/{room_name}/devices/{device_name}/report` | Get device report |

---

## Implementation

### 1. Application State

```rust
// src/state.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::smart_home::SmartHome;

pub type AppState = Arc<RwLock<SmartHome>>;

pub fn create_app_state() -> AppState {
    Arc::new(RwLock::new(SmartHome::new("My Smart Home")))
}
```

### 2. Request/Response Models

```rust
// src/models/requests.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub device_type: DeviceType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum DeviceType {
    #[serde(rename = "outlet")]
    Outlet { description: String },
    #[serde(rename = "thermometer")]
    Thermometer { temperature: f64 },
}
```

```rust
// src/models/responses.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub name: String,
    pub device_count: usize,
}

#[derive(Debug, Serialize)]
pub struct RoomsListResponse {
    pub rooms: Vec<RoomResponse>,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub name: String,
    pub device_type: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct DevicesListResponse {
    pub room_name: String,
    pub devices: Vec<DeviceResponse>,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub report: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
```

### 3. Error Handling

```rust
// src/api/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
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

    #[error("Internal server error")]
    InternalError,
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
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: message,
            code: status.as_u16(),
        });

        (status, body).into_response()
    }
}
```

### 4. Route Definitions

```rust
// src/api/routes.rs
use axum::{
    routing::{get, post, delete},
    Router,
};
use crate::state::AppState;
use super::handlers::{rooms, devices, reports};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Room routes
        .route("/api/v1/rooms", get(rooms::list_rooms))
        .route("/api/v1/rooms", post(rooms::create_room))
        .route("/api/v1/rooms/:room_name", get(rooms::get_room))
        .route("/api/v1/rooms/:room_name", delete(rooms::delete_room))
        // Device routes
        .route("/api/v1/rooms/:room_name/devices", get(devices::list_devices))
        .route("/api/v1/rooms/:room_name/devices", post(devices::create_device))
        .route("/api/v1/rooms/:room_name/devices/:device_name", get(devices::get_device))
        .route("/api/v1/rooms/:room_name/devices/:device_name", delete(devices::delete_device))
        // Report routes
        .route("/api/v1/report", get(reports::get_home_report))
        .route("/api/v1/rooms/:room_name/report", get(reports::get_room_report))
        .route("/api/v1/rooms/:room_name/devices/:device_name/report", get(reports::get_device_report))
        .with_state(state)
}
```

### 5. Room Handlers

```rust
// src/api/handlers/rooms.rs
use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    api::error::ApiError,
    models::{requests::CreateRoomRequest, responses::*},
    state::AppState,
};

pub async fn list_rooms(
    State(state): State<AppState>,
) -> Result<Json<RoomsListResponse>, ApiError> {
    let home = state.read().await;

    let rooms = home
        .rooms()
        .iter()
        .map(|(name, room)| RoomResponse {
            name: name.clone(),
            device_count: room.devices().len(),
        })
        .collect();

    Ok(Json(RoomsListResponse { rooms }))
}

pub async fn get_room(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<RoomResponse>, ApiError> {
    let home = state.read().await;

    let room = home
        .get_room(&room_name)
        .map_err(|_| ApiError::RoomNotFound(room_name.clone()))?;

    Ok(Json(RoomResponse {
        name: room_name,
        device_count: room.devices().len(),
    }))
}

pub async fn create_room(
    State(state): State<AppState>,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;

    // Check if room already exists
    if home.get_room(&payload.name).is_ok() {
        return Err(ApiError::RoomAlreadyExists(payload.name));
    }

    home.add_room(payload.name.clone())
        .map_err(|_| ApiError::InternalError)?;

    Ok(Json(MessageResponse {
        message: format!("Room '{}' created successfully", payload.name),
    }))
}

pub async fn delete_room(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;

    home.remove_room(&room_name)
        .map_err(|_| ApiError::RoomNotFound(room_name.clone()))?;

    Ok(Json(MessageResponse {
        message: format!("Room '{}' deleted successfully", room_name),
    }))
}
```

### 6. Device Handlers

```rust
// src/api/handlers/devices.rs
use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    api::error::ApiError,
    models::{requests::*, responses::*},
    state::AppState,
    smart_home::{SmartOutlet, SmartThermometer},
};

pub async fn list_devices(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<DevicesListResponse>, ApiError> {
    let home = state.read().await;

    let room = home
        .get_room(&room_name)
        .map_err(|_| ApiError::RoomNotFound(room_name.clone()))?;

    let devices = room
        .devices()
        .iter()
        .map(|(name, device)| DeviceResponse {
            name: name.clone(),
            device_type: device.device_type().to_string(),
            status: device.status(),
        })
        .collect();

    Ok(Json(DevicesListResponse {
        room_name,
        devices,
    }))
}

pub async fn get_device(
    State(state): State<AppState>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let home = state.read().await;

    let device = home
        .get_device(&room_name, &device_name)
        .map_err(|_| ApiError::DeviceNotFound(device_name.clone()))?;

    Ok(Json(DeviceResponse {
        name: device_name,
        device_type: device.device_type().to_string(),
        status: device.status(),
    }))
}

pub async fn create_device(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
    Json(payload): Json<CreateDeviceRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;

    // Check if room exists
    home.get_room(&room_name)
        .map_err(|_| ApiError::RoomNotFound(room_name.clone()))?;

    // Create device based on type
    let device = match payload.device_type {
        DeviceType::Outlet { description } => {
            SmartOutlet::new(&payload.name, &description).into()
        }
        DeviceType::Thermometer { temperature } => {
            SmartThermometer::new(&payload.name, temperature).into()
        }
    };

    home.add_device(&room_name, payload.name.clone(), device)
        .map_err(|_| ApiError::DeviceAlreadyExists(payload.name.clone()))?;

    Ok(Json(MessageResponse {
        message: format!("Device '{}' added to room '{}'", payload.name, room_name),
    }))
}

pub async fn delete_device(
    State(state): State<AppState>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<Json<MessageResponse>, ApiError> {
    let mut home = state.write().await;

    home.remove_device(&room_name, &device_name)
        .map_err(|_| ApiError::DeviceNotFound(device_name.clone()))?;

    Ok(Json(MessageResponse {
        message: format!("Device '{}' removed from room '{}'", device_name, room_name),
    }))
}
```

### 7. Report Handlers

```rust
// src/api/handlers/reports.rs
use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    api::error::ApiError,
    models::responses::ReportResponse,
    state::AppState,
    smart_home::Reporter,
};

pub async fn get_home_report(
    State(state): State<AppState>,
) -> Result<Json<ReportResponse>, ApiError> {
    let home = state.read().await;

    Ok(Json(ReportResponse {
        report: home.report(),
    }))
}

pub async fn get_room_report(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
) -> Result<Json<ReportResponse>, ApiError> {
    let home = state.read().await;

    let room = home
        .get_room(&room_name)
        .map_err(|_| ApiError::RoomNotFound(room_name))?;

    Ok(Json(ReportResponse {
        report: room.report(),
    }))
}

pub async fn get_device_report(
    State(state): State<AppState>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<Json<ReportResponse>, ApiError> {
    let home = state.read().await;

    let device = home
        .get_device(&room_name, &device_name)
        .map_err(|_| ApiError::DeviceNotFound(device_name))?;

    Ok(Json(ReportResponse {
        report: device.report(),
    }))
}
```

### 8. Main Entry Point

```rust
// src/main.rs
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod models;
mod state;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();

    // Create application state
    let state = state::create_app_state();

    // Build router
    let app = api::routes::create_router(state)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## Functional Tests

```rust
// tests/api_tests.rs
use reqwest::Client;
use serde_json::json;

const BASE_URL: &str = "http://localhost:3000/api/v1";

#[tokio::test]
async fn test_create_and_list_rooms() {
    let client = Client::new();

    // Create a room
    let response = client
        .post(format!("{}/rooms", BASE_URL))
        .json(&json!({ "name": "Living Room" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // List rooms
    let response = client
        .get(format!("{}/rooms", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["rooms"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_create_and_get_device() {
    let client = Client::new();

    // Create room first
    client
        .post(format!("{}/rooms", BASE_URL))
        .json(&json!({ "name": "Kitchen" }))
        .send()
        .await
        .unwrap();

    // Create device
    let response = client
        .post(format!("{}/rooms/Kitchen/devices", BASE_URL))
        .json(&json!({
            "name": "Main Outlet",
            "device_type": {
                "type": "outlet",
                "params": { "description": "Kitchen main outlet" }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Get device
    let response = client
        .get(format!("{}/rooms/Kitchen/devices/Main%20Outlet", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_home_report() {
    let client = Client::new();

    let response = client
        .get(format!("{}/report", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["report"].is_string());
}

#[tokio::test]
async fn test_delete_room() {
    let client = Client::new();

    // Create room
    client
        .post(format!("{}/rooms", BASE_URL))
        .json(&json!({ "name": "Temp Room" }))
        .send()
        .await
        .unwrap();

    // Delete room
    let response = client
        .delete(format!("{}/rooms/Temp%20Room", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify room is deleted
    let response = client
        .get(format!("{}/rooms/Temp%20Room", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_room_not_found() {
    let client = Client::new();

    let response = client
        .get(format!("{}/rooms/NonExistent", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}
```

---

## Running the Tests

1. Start the server in one terminal:
   ```bash
   cargo run --bin smart_home_server
   ```

2. Run the tests in another terminal:
   ```bash
   cargo test --test api_tests
   ```

Alternatively, use a test harness that spawns the server automatically:

```rust
// tests/common/mod.rs
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn spawn_test_server() -> String {
    let state = smart_home::state::create_app_state();
    let app = smart_home::api::routes::create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}
```

---

## API Usage Examples

### Create a Room

```bash
curl -X POST http://localhost:3000/api/v1/rooms \
  -H "Content-Type: application/json" \
  -d '{"name": "Living Room"}'
```

### List Rooms

```bash
curl http://localhost:3000/api/v1/rooms
```

### Add a Device

```bash
curl -X POST http://localhost:3000/api/v1/rooms/Living%20Room/devices \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Main Outlet",
    "device_type": {
      "type": "outlet",
      "params": {"description": "Living room main outlet"}
    }
  }'
```

### Get Home Report

```bash
curl http://localhost:3000/api/v1/report
```

---

## Summary

This implementation provides:

1. **RESTful API** with proper HTTP methods and status codes
2. **Thread-safe state** using `Arc<RwLock<SmartHome>>`
3. **Proper error handling** with custom error types
4. **Structured JSON responses** for all endpoints
5. **Functional tests** that verify API behavior
6. **Tracing/logging** for debugging and monitoring

The API integrates with the existing Smart Home library, exposing all required functionality through HTTP endpoints.
