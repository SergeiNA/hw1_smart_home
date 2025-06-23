use smart_home::info::Information;
use smart_home::smart_devices::{Celsius, DeviceType, OutletDevice, OutletState, Watt};
use smart_home::smart_home::SmartHome;
use smart_home::smart_room::SmartRoom;

fn main() {
    let bedroom = SmartRoom::new(
        "Bedroom".to_string(),
        vec![
            DeviceType::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
            DeviceType::new_outlet("Light Outlet".to_string(), OutletState::Off, 150 as Watt),
            DeviceType::new_thermometer("Electron thermometer".to_string(), 22.5 as Celsius),
        ],
    );
    let living_room = SmartRoom::new(
        "Living Room".to_string(),
        vec![
            DeviceType::new_outlet("Lighter".to_string(), OutletState::On, 100 as Watt),
            DeviceType::new_outlet("PC".to_string(), OutletState::On, 250 as Watt),
            DeviceType::new_thermometer("Electronic thermometer".to_string(), 22.5 as Celsius),
        ],
    );
    let kitchen = SmartRoom::new(
        "Kitchen Room".to_string(),
        vec![
            DeviceType::new_outlet(
                "Refrigerator Outlet".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            DeviceType::new_outlet("Teapot Outlet".to_string(), OutletState::Off, 150 as Watt),
            DeviceType::new_thermometer("Kitchen thermometer".to_string(), 20.0 as Celsius),
        ],
    );

    let mut home = SmartHome::new(
        "My Smart Home".to_string(),
        vec![bedroom, living_room, kitchen],
    );

    println!(
        "Home information before switch kitchen Teapot Outlet:\n{}\n\n\n",
        home.info()
    );

    {
        let device = home.get_room(2).get_device(1);
        let outlet = match device {
            DeviceType::OutletType(outlet) => outlet,
            _ => panic!("Expected OutletType"),
        };
        assert_eq!(outlet.state(), OutletState::Off);
        outlet.switch();
        assert_eq!(outlet.state(), OutletState::On);
    }

    println!(
        "Home information after switch kitchen Teapot Outlet:\n{}",
        home.info()
    );
}
