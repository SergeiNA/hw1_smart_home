use smart_home::report::Reporter;
use smart_home::smart_devices::{Celsius, Device, OutletState, Watt};
use smart_home::smart_room::SmartRoom;
use std::collections::HashMap;

fn main() {
    let thermometer =
        Device::new_thermometer("Living Room Thermometer".to_string(), 22.5 as Celsius);
    let outlet = Device::new_outlet(
        "Living Room Outlet".to_string(),
        OutletState::On,
        150 as Watt,
    );
    let room_thermometer = Device::new_thermometer("Room Thermometer".to_string(), 21.0 as Celsius);
    let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
    room.add_device("Thermometer".to_string(), room_thermometer);
    let reporter = Reporter::new()
        .add(&thermometer)
        .add(&outlet)
        .add(&room)
        .report();
    for report in reporter {
        println!("{}", report);
    }
}
