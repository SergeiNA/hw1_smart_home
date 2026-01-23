use crate::smart_devices::Device;

pub trait Information {
    fn name(&self) -> String;
    fn info(&self) -> String;
}

pub trait Subscriber {
    fn on_device_added(&mut self, device: &Device);
    fn on_device_removed(&mut self, device: &Device);
}
