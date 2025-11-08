use crate::smart_devices::Device;
use crate::traits::Information;
use crate::traits::Subscriber;

#[derive(Default)]
pub struct DefaultSubscriber;

impl Subscriber for DefaultSubscriber {
    fn on_device_added(&mut self, device: &Device) {
        println!("[default sub] Device added: {}", device.name());
    }

    fn on_device_removed(&mut self, device: &Device) {
        println!("[default sub] Device removed: {}", device.name());
    }
}

pub enum DeviceEvent<'a> {
    Added(&'a Device),
    Removed(&'a Device),
}

impl<F> Subscriber for F
where
    F: for<'a> FnMut(DeviceEvent),
{
    fn on_device_added(&mut self, device: &Device) {
        println!("[closure sub] on device: {}", device.name());
        (self)(DeviceEvent::Added(device));
    }
    fn on_device_removed(&mut self, device: &Device) {
        println!("[use closure sub] on device: {}", device.name());
        (self)(DeviceEvent::Removed(device));
    }
}

mod tests {
    use super::*;
    use crate::smart_devices::{Device, OutletState, Watt};

    #[test]
    fn test_default_subscriber() {
        let mut subscriber = DefaultSubscriber::default();
        let outlet = Device::new_outlet("Test Outlet".to_string(), OutletState::On, 100 as Watt);
        subscriber.on_device_added(&outlet);
        subscriber.on_device_removed(&outlet);
    }

    #[test]
    fn test_closure_subscriber() {
        let mut closure_subscriber = |event: DeviceEvent| match event {
            DeviceEvent::Added(dev) => println!("[closure] Device added: {}", dev.name()),
            DeviceEvent::Removed(dev) => println!("[closure] Device removed: {}", dev.name()),
        };

        let outlet = Device::new_outlet("Test Outlet".to_string(), OutletState::On, 100 as Watt);
        closure_subscriber.on_device_added(&outlet);
        closure_subscriber.on_device_removed(&outlet);
    }
}
