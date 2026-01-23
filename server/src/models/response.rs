use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RoomResponse {
    pub name: String,
    pub devices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RoomListResponse {
    pub rooms: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HomeReportResponse {
    pub name: String,
    pub report: String,
}

#[derive(Debug, Serialize)]
pub struct RoomReportResponse {
    pub name: String,
    pub report: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceReportResponse {
    pub name: String,
    pub report: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
