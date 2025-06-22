pub trait SmartDevice {
    fn device_name(&self) -> String;
    fn device_info(&self) -> String;
}
