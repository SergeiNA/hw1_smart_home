use smart_home::create_home;
use smart_home::create_room;
use smart_home::simulators::outlet::OutletSimulatorConfig;
use smart_home::simulators::spawner::DeviceSimulatorSpawner;
use smart_home::simulators::thermometer::{TemperaturePattern, ThermometerSimulatorConfig};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::thermometer_remote::ThermometerRemote;
use smart_home::smart_devices::types::OutletState;
use smart_home::smart_devices::{Device, OutletDevice, Watt};
use smart_home::smart_home::SmartHome;
use smart_home::smart_room::SmartRoom;
use smart_home::traits::Information;
use std::thread;
use std::time::Duration;

fn main() {
    // Configure and spawn remote device simulators
    let mut spawner = DeviceSimulatorSpawner::default();

    let living_room_outlet_simulator_addr = spawner
        .spawn_outlet_simulator(
            "TV outlet".to_string(),
            OutletSimulatorConfig::new("127.0.0.1:9010", 220 as Watt),
        )
        .unwrap_or_else(|e| {
            eprintln!("Failed to spawn outlet simulator: {}", e);
            panic!("Failed to spawn outlet simulator: {}", e);
        });

    let bed_room_outlet_simulator_addr = spawner
        .spawn_outlet_simulator(
            "Lamp outlet".to_string(),
            OutletSimulatorConfig::new("127.0.0.1:9020", 120 as Watt),
        )
        .unwrap_or_else(|e| {
            eprintln!("Failed to spawn outlet simulator: {}", e);
            panic!("Failed to spawn outlet simulator: {}", e);
        });

    let kitchen_thermometer_simulator_target_addr = {
        let config = ThermometerSimulatorConfig::new(
            "Bosh D1431",
            "127.0.0.1:9030".to_string(),
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::RandomWalk {
            step: 0.5,
            min: 15.0,
            max: 25.0,
        });

        let kitchen_thermometer_simulator_target_addr = spawner
            .spawn_thermometer_simulator("Kitchen thermometer".to_string(), config)
            .unwrap_or_else(|e| {
                eprintln!("Failed to spawn thermometer simulator: {}", e);
                panic!("Failed to spawn thermometer simulator: {}", e);
            });
        kitchen_thermometer_simulator_target_addr
    };

    let bedroom_thermometer_simulator_target_addr = {
        let config = ThermometerSimulatorConfig::new(
            "ABC D1431",
            "127.0.0.1:9021".to_string(),
            Duration::from_millis(150),
        )
        .with_pattern(TemperaturePattern::SineWave {
            center: 22.0,
            amplitude: 4.,
            period_secs: 1.,
        });

        let kitchen_thermometer_simulator_target_addr = spawner
            .spawn_thermometer_simulator("Bedroom thermometer".to_string(), config)
            .unwrap_or_else(|e| {
                eprintln!("Failed to spawn thermometer simulator: {}", e);
                panic!("Failed to spawn thermometer simulator: {}", e);
            });
        kitchen_thermometer_simulator_target_addr
    };

    // Create smart home instance connecting to remote simulators

    // Create rooms

    // Living room
    let remote_outlet = OutletRemote::new(
        "Living Room Lamp".to_string(),
        living_room_outlet_simulator_addr,
    )
    .expect("Failed to create remote outlet");

    let living_room = create_room!(
        "Living Room",
        "Living Room Lamp" => Device::from(remote_outlet),
    );

    // Bedroom
    let remote_bedroom_outlet =
        OutletRemote::new("Bedroom Lamp".to_string(), bed_room_outlet_simulator_addr)
            .expect("Failed to create remote outlet");

    let bedroom_thermometer = ThermometerRemote::new(
        "Bedroom Thermometer".to_string(),
        bedroom_thermometer_simulator_target_addr,
    )
    .expect("Failed to create remote thermometer");

    let bedroom = create_room!(
        "Bedroom",
        "Bedroom Lamp" => Device::from(remote_bedroom_outlet),
        "Bedroom Thermometer" => Device::from(bedroom_thermometer),
    );

    // Kitchen
    let kitchen_thermometer = ThermometerRemote::new(
        "Kitchen Thermometer".to_string(),
        kitchen_thermometer_simulator_target_addr,
    )
    .expect("Failed to create remote thermometer");

    let kitchen = create_room!(
        "Kitchen",
        "Kitchen Thermometer" => Device::from(kitchen_thermometer),
    );

    thread::sleep(Duration::from_millis(300));

    let mut home = create_home!(
        "My Smart Home",
        {"Living room", living_room},
        {"Bedroom", bedroom},
        {"Kitchen", kitchen},
    );

    println!("Current home information:\n{}\n\n\n", home.info());

    thread::sleep(Duration::from_secs(1));

    println!("After 3 sec home information:\n{}\n\n\n", home.info());

    // Switch Bedroom outlet
    {
        let device = home
            .get_room("Bedroom")
            .and_then(|room| room.get_device("Bedroom Lamp"));
        let outlet = match device {
            Some(Device::OutletTypeRemote(outlet)) => outlet,
            _ => panic!("Expected OutletType"),
        };
        let state = match outlet.state() {
            Ok(s) => s,
            Err(e) => panic!("Failed to get outlet state: {}", e),
        };
        assert_eq!(state, OutletState::Off);

        match outlet.switch() {
            Ok(s) => s,
            Err(e) => panic!("Failed to get outlet state: {}", e),
        };
        let state = match outlet.state() {
            Ok(s) => s,
            Err(e) => panic!("Failed to get outlet state: {}", e),
        };
        assert_eq!(state, OutletState::On);
    }

    // Switch Living room outlet
    {
        let device = home
            .get_room("Living room")
            .and_then(|room| room.get_device("Living Room Lamp"));
        let outlet = match device {
            Some(Device::OutletTypeRemote(outlet)) => outlet,
            _ => panic!("Expected OutletType"),
        };
        let state = match outlet.state() {
            Ok(s) => s,
            Err(e) => panic!("Failed to get outlet state: {}", e),
        };
        assert_eq!(state, OutletState::Off);

        match outlet.switch() {
            Ok(s) => s,
            Err(e) => panic!("Failed to get outlet state: {}", e),
        };
        let state = match outlet.state() {
            Ok(s) => s,
            Err(e) => panic!("Failed to get outlet state: {}", e),
        };
        assert_eq!(state, OutletState::On);
    }

    println!("Turn On all outlets:\n{}\n\n\n", home.info());
}
