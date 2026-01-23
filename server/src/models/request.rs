use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddRoomRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AddDeviceRequest {
    pub name: String,
    pub device_type: DeviceType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum DeviceType {
    #[serde(rename = "outlet")]
    Outlet { power: u16 },
    #[serde(rename = "thermometer")]
    Thermometer { temperature: f64 },
}
