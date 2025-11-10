use smart_home::smart_devices::{Celsius, Device, OutletState, Watt};
use smart_home::smart_room::SmartRoom;
use smart_home::subscriber::DefaultSubscriber;
use smart_home::subscriber::DeviceEvent;
use smart_home::traits::Information;
use std::collections::HashMap;

fn main() {
    let thermometer =
        Device::new_thermometer("Living Room Thermometer".to_string(), 22.5 as Celsius);
    let outlet = Device::new_outlet(
        "Living Room Outlet".to_string(),
        OutletState::On,
        150 as Watt,
    );
    let subscriber = DefaultSubscriber;

    let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
    room.subscribe(subscriber);
    room.subscribe(|device: DeviceEvent| match device {
        DeviceEvent::Added(dev) => println!("Device added to cache: {}", dev.name()),
        DeviceEvent::Removed(dev) => println!("Device removed from cache: {}", dev.name()),
    });
    room.add_device("Living Room Thermometer".to_string(), thermometer);
    room.add_device("Living Room Outlet".to_string(), outlet);
    room.remove_device("Bedroom Room Thermometer");
    room.remove_device("Living Room Thermometer");
}
