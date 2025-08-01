use smart_home::create_home;
use smart_home::create_room;
use smart_home::smart_devices::{Celsius, Device, OutletDevice, OutletState, Watt};
use smart_home::smart_home::SmartHome;
use smart_home::smart_room::SmartRoom;
use smart_home::traits::Information;
use std::error::Error;

fn log_error(message: &dyn Error) {
    eprintln!();
    eprintln!("Error: {}", message);
    eprintln!();
}

fn main() {
    // Basic usage example of the smart home system
    let mut home = create_home!(
        "My Smart Home",
        {
            "Bedroom",
            create_room!(
                "Bedroom",
                "Attached Outlet" => Device::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
                "Light Outlet" => Device::new_outlet("Light Outlet".to_string(), OutletState::Off, 150 as Watt),
                "Electron thermometer" => Device::new_thermometer("Electron thermometer".to_string(), 22.5 as Celsius)
            )
        },
        {
            "Living Room",
            create_room!(
                "Living Room",
                "Lighter" => Device::new_outlet("Lighter".to_string(), OutletState::On, 100 as Watt),
                "PC" => Device::new_outlet("PC".to_string(), OutletState::On, 250 as Watt),
                "Electronic thermometer" => Device::new_thermometer("Electronic thermometer".to_string(), 22.5 as Celsius)
            )
        },
        {
            "Kitchen Room",
            create_room!(
                "Kitchen Room",
                "Refrigerator Outlet" => Device::new_outlet("Refrigerator Outlet".to_string(), OutletState::On, 100 as Watt),
                "Teapot Outlet" => Device::new_outlet("Teapot Outlet".to_string(), OutletState::Off, 150 as Watt),
                "Kitchen thermometer" => Device::new_thermometer("Kitchen thermometer".to_string(), 20.0 as Celsius)
            )
        }
    );

    println!(
        "Home information before switch kitchen Teapot Outlet:\n{}\n\n\n",
        home.info()
    );

    // Switch outlet check
    {
        let device = home
            .get_room("Kitchen Room")
            .and_then(|room| room.get_device("Teapot Outlet"));
        let outlet = match device {
            Some(Device::OutletType(outlet)) => outlet,
            _ => panic!("Expected OutletType"),
        };
        assert_eq!(outlet.state(), OutletState::Off);
        outlet.switch();
        assert_eq!(outlet.state(), OutletState::On);
    }

    {
        let device = home
            .get_room("Living Room")
            .and_then(|room| room.get_device("Lighter"));
        let outlet = match device {
            Some(Device::OutletType(outlet)) => outlet,
            _ => panic!("Expected OutletType"),
        };
        assert_eq!(outlet.state(), OutletState::On);
        outlet.turn_off();
        assert_eq!(outlet.state(), OutletState::Off);
    }

    println!(
        "Home information after switch kitchen Teapot Outlet:\n{}",
        home.info()
    );

    // Access room/device information and error handling
    assert!(home.device("Living Room", "PC").is_ok());
    assert!(home.device("Kitchen Room", "Teapot Outlet").is_ok());
    assert!(home.device("Kitchen Room", "PC").is_err());

    log_error(&home.device("Kitchen Room", "PC").unwrap_err());

    // Remove, add room
    {
        // Remove a room
        home.remove_room("Bedroom");
        assert!(home.get_room("Bedroom").is_none());

        // Add a new room
        let new_room = create_room!(
            "New Room",
            "New Outlet" => Device::new_outlet("New Outlet".to_string(), OutletState::On, 200 as Watt)
        );

        home.add_room(new_room);

        // Check if the new room is added
        assert!(home.get_room("New Room").is_some());

        home.remove_room("New Room");
        assert!(home.get_room("New Room").is_none());
    }
}
