use smart_home::smart_room::SmartRoom;

fn main() {
    let room = SmartRoom::new(
        "Living Room".to_string(),
        vec![
            smart_home::smart_devices::DeviceType::new_outlet(
                "Smart Outlet lighter".to_string(),
                smart_home::smart_devices::OutletState::On,
                100,
            ),
            smart_home::smart_devices::DeviceType::new_outlet(
                "Smart Outlet PC".to_string(),
                smart_home::smart_devices::OutletState::On,
                250,
            ),
        ],
    );
    println!("{:?}", room);
}
