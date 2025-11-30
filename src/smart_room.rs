use crate::smart_devices::Device;
use crate::traits::Information;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::Display;
use std::string::String;

#[derive(Debug)]
pub struct SmartRoom {
    name: String,
    devices: HashMap<String, Device>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessError {
    pub message: String,
}

impl Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccessError: {}", self.message)
    }
}

impl Error for AccessError {}

impl Information for SmartRoom {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn info(&self) -> String {
        let mut sorted_devices: BTreeMap<&String, &Device> = self.devices.iter().collect();
        let enumerated_devices: Vec<String> = sorted_devices
            .iter_mut()
            .enumerate()
            .map(|(i, d)| format!("[{}]: {}", i, d.1.info()))
            .collect();
        format!(
            "\nSmart Room: {}:\n Total devices: {}\n  {}",
            self.name,
            enumerated_devices.len(),
            enumerated_devices.join("\n  --------------------------------------\n  ")
        )
    }
}

impl SmartRoom {
    /// Creates a new SmartRoom with the given name and devices.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the smart room.
    /// * `devices` - A HashMap containing devices with their identifying keys.
    ///
    /// # Returns
    ///
    /// A new SmartRoom instance.
    pub fn new(name: String, devices: HashMap<String, Device>) -> Self {
        SmartRoom { name, devices }
    }

    /// Returns an immutable reference to the device with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the device in the internal device map.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the device if found, or `None` if not found.
    pub fn view_device(&self, key: &str) -> Option<&Device> {
        self.devices.get(key)
    }

    /// Returns a mutable reference to the device with the given key,
    /// allowing the caller to modify the device.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the device in the internal device map.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the device if found, or `None` if not found.
    pub fn get_device(&mut self, key: &str) -> Option<&mut Device> {
        self.devices.get_mut(key)
    }

    /// Adds a new device to the room with the specified key.
    /// If a device with the same key already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `key` - The unique identifier for the device.
    /// * `device` - The device to be added to the room.
    pub fn add_device(&mut self, key: String, device: Device) {
        self.devices.insert(key, device);
    }

    /// Removes a device from the room by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the device to be removed.
    ///
    /// # Returns
    ///
    /// An `Option` containing the removed device if it was found, or `None` if not found.
    pub fn remove_device(&mut self, key: &str) -> Option<Device> {
        self.devices.remove(key)
    }
}

/// Trait for types that provide controlled access to devices.
///
/// This trait defines a mechanism to safely access devices by their identifier
/// with proper error handling when a device is not found.
pub trait AccessDevice {
    /// Attempts to access a device by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The unique identifier for the device to access.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// * `Ok(&Device)` - A reference to the requested device if found
    /// * `Err(AccessError)` - An error if the device could not be found
    fn access_device(&self, key: &str) -> Result<&Device, AccessError>;
}

impl AccessDevice for SmartRoom {
    /// Provides controlled access to devices in the smart room.
    ///
    /// This implementation uses the internal `view` method to get a reference to a device,
    /// and wraps the result in a `Result` type for better error handling.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the device in the room's device collection.
    ///
    /// # Returns
    ///
    /// * `Ok(&Device)` - A reference to the device if found
    /// * `Err(AccessError)` - An error with a descriptive message if the device was not found
    fn access_device(&self, key: &str) -> Result<&Device, AccessError> {
        self.view_device(key).ok_or(AccessError {
            message: format!(
                "Device with the name '{}' not found in the room '{}'",
                key,
                self.name.clone()
            ),
        })
    }
}

#[macro_export]
macro_rules! create_room {
    ($name:expr, $( $key:expr => $value:expr ),* $(,)? ) => {{
        let devices = std::collections::HashMap::from(
          [
            $( ($key.to_string(), $value) ),*
          ]
        );
        SmartRoom::new($name.to_string(), devices)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::types::OutletState;
    use crate::smart_devices::{Celsius, OutletDevice, TemperatureSensor, Watt};

    const TEST_DEFAULT_DEVICE: Device = Device::Empty;

    #[test]
    fn smart_room_create_empty_test() {
        let room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
    }

    #[test]
    fn smart_room_view_index_out_of_bounds_test() {
        let room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
        assert!(room.view_device("Some device").is_none());
    }

    #[test]
    fn smart_room_get_index_out_of_bounds_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
        assert!(room.get_device("Some device").is_none());
    }

    #[test]
    fn smart_room_macro_create_test() {
        {
            let room = create_room!(
                "Living Room",
                "Lighter" => Device::new_outlet("Lighter".to_string(), OutletState::On, 100 as Watt),
                "PC" => Device::new_outlet("PC".to_string(), OutletState::On, 250 as Watt),
                "Electronic thermometer" => Device::new_thermometer("Electronic thermometer".to_string(), 22.5 as Celsius)
            );
            assert_eq!(room.name(), "Living Room");
            assert_eq!(room.devices.len(), 3);
        }
        {
            let room = create_room!("Kitchen",);
            assert_eq!(room.name(), "Kitchen");
            assert_eq!(room.devices.len(), 0);
        }
    }

    #[test]
    fn smart_room_view_test() {
        let room = create_room!(
            "Living Room",
            "Lighter" => Device::new_outlet("Lighter".to_string(), OutletState::On, 100 as Watt),
            "PC" => Device::new_outlet("PC".to_string(), OutletState::On, 250 as Watt),
            "Electronic thermometer" => Device::new_thermometer("Electronic thermometer".to_string(), 22.5 as Celsius)
        );

        assert_eq!(room.devices.len(), 3);
        assert!(room.view_device("Some device").is_none());
        assert_eq!(room.view_device("Lighter").is_some(), true);
        assert_eq!(
            room.view_device("Lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "Lighter"
        );
        assert_eq!(
            room.view_device("Lighter")
                .expect("Failed to get device 'Lighter'")
                .info(),
            "Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(room.view_device("PC").is_some(), true);
        assert_eq!(
            room.view_device("PC")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "PC"
        );
        assert_eq!(
            room.view_device("PC")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(room.view_device("Electronic thermometer").is_some(), true);
        assert_eq!(
            room.view_device("Electronic thermometer")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "Electronic thermometer"
        );
        assert_eq!(
            room.view_device("Electronic thermometer")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Thermometer: Electronic thermometer - Current Temperature: 22.50°C"
        );

        let outlet_lighter: &Device = room.view_device("Lighter").unwrap_or(&TEST_DEFAULT_DEVICE);
        let outlet_pc: &Device = room.view_device("PC").unwrap_or(&TEST_DEFAULT_DEVICE);
        let thermometer: &Device = room
            .view_device("Electronic thermometer")
            .unwrap_or(&TEST_DEFAULT_DEVICE);

        assert_eq!(outlet_lighter.name(), "Lighter");
        assert_eq!(
            outlet_lighter.info(),
            "Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(outlet_pc.name(), "PC");
        assert_eq!(
            outlet_pc.info(),
            "Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(thermometer.name(), "Electronic thermometer");
        assert_eq!(
            thermometer.info(),
            "Thermometer: Electronic thermometer - Current Temperature: 22.50°C"
        );
    }

    #[test]
    fn smart_room_add_one_device_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        let outlet = Device::new_outlet("Smart Outlet".to_string(), OutletState::On, 150 as Watt);
        room.add_device("Smart Outlet".to_string(), outlet);
        assert_eq!(room.devices.len(), 1);
        assert_eq!(
            room.view_device("Smart Outlet")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "Smart Outlet"
        );
    }

    #[test]
    fn smart_room_add_many_devices_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        room.add_device(
            "Smart Outlet lighter".to_string(),
            Device::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
        );
        room.add_device(
            "Smart Outlet PC".to_string(),
            Device::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        );
        room.add_device(
            "Smart Thermometer".to_string(),
            Device::new_thermometer("Smart Thermometer".to_string(), 22.5 as Celsius),
        );
        assert_eq!(room.devices.len(), 3);
        assert_eq!(
            room.view_device("Smart Outlet lighter").unwrap().name(),
            "Smart Outlet lighter"
        );
        assert_eq!(
            room.view_device("Smart Outlet PC").unwrap().name(),
            "Smart Outlet PC"
        );
        assert_eq!(
            room.view_device("Smart Thermometer").unwrap().name(),
            "Smart Thermometer"
        );

        let expected = r#"
Smart Room: Living Room:
 Total devices: 3
  [0]: Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Smart Outlet: Smart Outlet lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Thermometer: Smart Thermometer - Current Temperature: 22.50°C"#;
        assert_eq!(room.info(), expected);
    }

    #[test]
    fn smart_room_remove_one_device_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        let outlet = Device::new_outlet("Smart Outlet".to_string(), OutletState::On, 150 as Watt);
        room.add_device("Smart Outlet".to_string(), outlet);

        assert!(room.remove_device("Not existing device").is_none());
        assert_eq!(room.devices.len(), 1);

        let removed_device = room
            .remove_device("Smart Outlet")
            .unwrap_or(TEST_DEFAULT_DEVICE);
        assert_eq!(removed_device.name(), "Smart Outlet");
        assert_eq!(room.devices.len(), 0);

        assert!(room.remove_device("Smart Outlet").is_none());
    }

    #[test]
    fn smart_room_remove_many_devices_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        room.add_device(
            "Smart Outlet lighter".to_string(),
            Device::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
        );
        room.add_device(
            "Smart Outlet PC".to_string(),
            Device::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        );
        room.add_device(
            "Smart Thermometer".to_string(),
            Device::new_thermometer("Smart Thermometer".to_string(), 22.5 as Celsius),
        );

        assert_eq!(room.devices.len(), 3);

        let removed_device = room
            .remove_device("Smart Outlet lighter")
            .unwrap_or(TEST_DEFAULT_DEVICE);
        assert_eq!(removed_device.name(), "Smart Outlet lighter");
        assert_eq!(room.devices.len(), 2);

        let removed_device = room
            .remove_device("Smart Outlet PC")
            .unwrap_or(TEST_DEFAULT_DEVICE);
        assert_eq!(removed_device.name(), "Smart Outlet PC");
        assert_eq!(room.devices.len(), 1);

        let removed_device = room
            .remove_device("Smart Thermometer")
            .unwrap_or(TEST_DEFAULT_DEVICE);
        assert_eq!(removed_device.name(), "Smart Thermometer");
        assert_eq!(room.devices.len(), 0);

        assert!(room.remove_device("Smart Outlet lighter").is_none());
        assert!(room.remove_device("Smart Outlet PC").is_none());
        assert!(room.remove_device("Smart Thermometer").is_none());
    }

    #[test]
    fn smart_room_ref_device_test() {
        let room = create_room!(
            "Living Room",
            "Smart Outlet lighter" => Device::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            "Smart Outlet PC" => Device::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        );
        let device_0 = room
            .view_device("Smart Outlet lighter")
            .unwrap_or(&TEST_DEFAULT_DEVICE);
        let outlet_0 = match device_0 {
            Device::OutletTypeMock(o) => o,
            _ => panic!("Expected OutletType"),
        };

        assert_eq!(outlet_0.name(), "Smart Outlet lighter");
        assert_eq!(
            outlet_0.state().expect("Failed to get outlet state"),
            OutletState::On
        );
        assert_eq!(
            outlet_0
                .power_usage()
                .expect("Failed to get outlet power usage"),
            100 as Watt
        );
    }

    #[test]
    fn smart_room_get_test() {
        let mut default_device = Device::Empty;
        let mut room = create_room!(
            "Living Room",
            "Smart Outlet lighter" => Device::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            "Smart Outlet PC" => Device::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        );

        assert_eq!(room.devices.len(), 2);
        assert_eq!(
            room.view_device("Smart Outlet lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: Smart Outlet lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(
            room.view_device("Smart Outlet PC")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt"
        );

        {
            let outlet_device: &mut Device = room
                .get_device("Smart Outlet lighter")
                .unwrap_or(&mut default_device);
            let outlet = match outlet_device {
                Device::OutletTypeMock(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(
                outlet.state().expect("Failed to get outlet state"),
                OutletState::On
            );
            outlet.switch().expect("Failed to switch outlet state");
            assert_eq!(
                outlet.state().expect("Failed to get outlet state"),
                OutletState::Off
            );
        }
        assert_eq!(
            room.view_device("Smart Outlet lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: Smart Outlet lighter - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            room.view_device("Smart Outlet PC")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt"
        );

        {
            let outlet_device: &mut Device = room
                .get_device("Smart Outlet PC")
                .unwrap_or(&mut default_device);
            let outlet = match outlet_device {
                Device::OutletTypeMock(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(
                outlet.state().expect("Failed to get outlet state"),
                OutletState::On
            );
            outlet.switch().expect("Failed to switch outlet state");
            assert_eq!(
                outlet.state().expect("Failed to get outlet state"),
                OutletState::Off
            );
        }
        assert_eq!(
            room.view_device("Smart Outlet lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: Smart Outlet lighter - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            room.view_device("Smart Outlet PC")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .info(),
            "Smart Outlet: Smart Outlet PC - Current State: Off, Power Usage: 0 Watt"
        );
    }

    #[test]
    fn smart_room_access_device_test() {
        let room = create_room!(
            "Living Room",
            "Smart Outlet lighter" => Device::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            "Smart Outlet PC" => Device::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        );

        assert_eq!(
            room.access_device("Smart Outlet lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "Smart Outlet lighter"
        );
        assert_eq!(
            room.access_device("Smart Outlet PC")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "Smart Outlet PC"
        );
        assert!(room.access_device("Non-existing device").is_err());
        let err = room.access_device("Non-existing device").unwrap_err();
        assert_eq!(
            err.to_string(),
            "AccessError: Device with the name 'Non-existing device' not found in the room 'Living Room'"
        );
    }

    // Tests with remote devices
    #[test]
    fn smart_room_with_remote_outlet_test() {
        use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
        use crate::smart_devices::outlet_remote::OutletRemote;
        use std::thread;
        use std::time::Duration;

        // Spawn an outlet simulator
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 200 as Watt);
        let simulator = OutletSimulator::spawn(config).expect("Failed to spawn simulator");
        let addr = simulator.address().to_string();

        thread::sleep(Duration::from_millis(100));

        // Create a remote outlet connected to the simulator
        let remote_outlet = OutletRemote::new("Remote Lamp".to_string(), addr)
            .expect("Failed to create remote outlet");

        // Create a room and add the remote outlet
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        room.add_device("Remote Lamp".to_string(), Device::from(remote_outlet));

        assert_eq!(room.devices.len(), 1);
        assert_eq!(
            room.view_device("Remote Lamp").unwrap().name(),
            "Remote Lamp"
        );

        // Verify initial state
        let device = room.view_device("Remote Lamp").unwrap();
        match device {
            Device::OutletTypeRemote(outlet) => {
                assert_eq!(outlet.state().unwrap(), OutletState::Off);
                assert_eq!(outlet.power_usage().unwrap(), 0);
            }
            _ => panic!("Expected OutletTypeRemote"),
        }
    }

    #[test]
    fn smart_room_with_remote_outlet_operations_test() {
        use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
        use crate::smart_devices::outlet_remote::OutletRemote;
        use std::thread;
        use std::time::Duration;

        // Spawn simulator
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 150 as Watt);
        let simulator = OutletSimulator::spawn(config).expect("Failed to spawn simulator");
        let addr = simulator.address().to_string();

        thread::sleep(Duration::from_millis(100));

        let remote_outlet = OutletRemote::new("Remote Device".to_string(), addr)
            .expect("Failed to create remote outlet");

        let mut room = SmartRoom::new("Kitchen".to_string(), HashMap::new());
        room.add_device("Remote Device".to_string(), Device::from(remote_outlet));

        // Get mutable reference and turn on the outlet
        {
            let device = room.get_device("Remote Device").unwrap();
            match device {
                Device::OutletTypeRemote(outlet) => {
                    outlet.turn_on().expect("Failed to turn on");
                    thread::sleep(Duration::from_millis(50));
                    assert_eq!(outlet.state().unwrap(), OutletState::On);
                    assert_eq!(outlet.power_usage().unwrap(), 150);
                }
                _ => panic!("Expected OutletTypeRemote"),
            }
        }

        // Verify state persists
        let device = room.view_device("Remote Device").unwrap();
        match device {
            Device::OutletTypeRemote(outlet) => {
                assert_eq!(outlet.state().unwrap(), OutletState::On);
                assert_eq!(outlet.power_usage().unwrap(), 150);
            }
            _ => panic!("Expected OutletTypeRemote"),
        }

        // Turn off
        {
            let device = room.get_device("Remote Device").unwrap();
            match device {
                Device::OutletTypeRemote(outlet) => {
                    outlet.turn_off().expect("Failed to turn off");
                    thread::sleep(Duration::from_millis(50));
                    assert_eq!(outlet.state().unwrap(), OutletState::Off);
                    assert_eq!(outlet.power_usage().unwrap(), 0);
                }
                _ => panic!("Expected OutletTypeRemote"),
            }
        }
    }

    #[test]
    fn smart_room_with_remote_thermometer_test() {
        use crate::simulators::thermometer::{
            TemperaturePattern, ThermometerSimulator, ThermometerSimulatorConfig,
        };
        use crate::smart_devices::thermometer_remote::ThermometerRemote;
        use std::thread;
        use std::time::Duration;

        let udp_addr = "127.0.0.1:40001".to_string();

        // Create remote thermometer first
        let remote_thermo = ThermometerRemote::new("Remote Sensor".to_string(), udp_addr.clone())
            .expect("Failed to create remote thermometer");

        thread::sleep(Duration::from_millis(100));

        // Spawn simulator that sends to this address
        let config = ThermometerSimulatorConfig::new("TempSim", udp_addr, Duration::from_secs(1))
            .with_pattern(TemperaturePattern::Constant(23.5));

        let _simulator =
            ThermometerSimulator::spawn(config).expect("Failed to spawn thermometer simulator");

        // Create room and add the remote thermometer
        let mut room = SmartRoom::new("Bedroom".to_string(), HashMap::new());
        room.add_device("Remote Sensor".to_string(), Device::from(remote_thermo));

        assert_eq!(room.devices.len(), 1);
        assert_eq!(
            room.view_device("Remote Sensor").unwrap().name(),
            "Remote Sensor"
        );

        // Wait for temperature update
        thread::sleep(Duration::from_millis(1500));

        // Verify temperature was received
        let device = room.view_device("Remote Sensor").unwrap();
        match device {
            Device::ThermometerTypeRemote(thermo) => {
                let temp = thermo.current_temperature();
                assert!((temp - 23.5).abs() < 0.1, "Expected ~23.5, got {}", temp);
            }
            _ => panic!("Expected ThermometerTypeRemote"),
        }
    }

    #[test]
    fn smart_room_with_multiple_remote_devices_test() {
        use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
        use crate::simulators::thermometer::{
            TemperaturePattern, ThermometerSimulator, ThermometerSimulatorConfig,
        };
        use crate::smart_devices::outlet_remote::OutletRemote;
        use crate::smart_devices::thermometer_remote::ThermometerRemote;
        use std::thread;
        use std::time::Duration;

        // Spawn outlet simulator
        let outlet_config = OutletSimulatorConfig::new("127.0.0.1:0", 100 as Watt);
        let outlet_sim =
            OutletSimulator::spawn(outlet_config).expect("Failed to spawn outlet simulator");
        let outlet_addr = outlet_sim.address().to_string();

        // Setup thermometer
        let thermo_addr = "127.0.0.1:40010".to_string();
        let remote_thermo =
            ThermometerRemote::new("Bedroom Sensor".to_string(), thermo_addr.clone())
                .expect("Failed to create remote thermometer");

        thread::sleep(Duration::from_millis(100));

        let thermo_config =
            ThermometerSimulatorConfig::new("ThermoSim", thermo_addr, Duration::from_secs(1))
                .with_pattern(TemperaturePattern::Constant(22.0));

        let _thermo_sim = ThermometerSimulator::spawn(thermo_config)
            .expect("Failed to spawn thermometer simulator");

        // Create remote outlet
        let remote_outlet = OutletRemote::new("Bedroom Lamp".to_string(), outlet_addr)
            .expect("Failed to create remote outlet");

        // Create room with both devices
        let room = create_room!(
            "Bedroom",
            "Bedroom Lamp" => Device::from(remote_outlet),
            "Bedroom Sensor" => Device::from(remote_thermo),
        );

        assert_eq!(room.devices.len(), 2);
        assert!(room.view_device("Bedroom Lamp").is_some());
        assert!(room.view_device("Bedroom Sensor").is_some());

        // Wait for thermometer update
        thread::sleep(Duration::from_millis(1500));

        // Verify both devices work
        let outlet_device = room.view_device("Bedroom Lamp").unwrap();
        match outlet_device {
            Device::OutletTypeRemote(outlet) => {
                assert_eq!(outlet.state().unwrap(), OutletState::Off);
            }
            _ => panic!("Expected OutletTypeRemote"),
        }

        let thermo_device = room.view_device("Bedroom Sensor").unwrap();
        match thermo_device {
            Device::ThermometerTypeRemote(thermo) => {
                let temp = thermo.current_temperature();
                assert!((temp - 22.0).abs() < 0.1, "Expected ~22.0, got {}", temp);
            }
            _ => panic!("Expected ThermometerTypeRemote"),
        }
    }

    #[test]
    fn smart_room_with_remote_devices_from_spawner_test() {
        use crate::simulators::outlet::OutletSimulatorConfig;
        use crate::simulators::spawner::DeviceSimulatorSpawner;
        use crate::simulators::thermometer::{TemperaturePattern, ThermometerSimulatorConfig};
        use crate::smart_devices::outlet_remote::OutletRemote;
        use crate::smart_devices::thermometer_remote::ThermometerRemote;
        use std::thread;
        use std::time::Duration;

        let mut simulator_spawner = DeviceSimulatorSpawner::default();

        // Spawn outlet simulator
        let outlet_config = OutletSimulatorConfig::new("127.0.0.1:0", 100 as Watt);
        let outlet_addr = simulator_spawner
            .spawn_outlet_simulator("Bedroom Lamp".to_string(), outlet_config)
            .unwrap();

        // Setup thermometer
        let thermo_addr = "127.0.0.1:40011".to_string();
        let remote_thermo =
            ThermometerRemote::new("Bedroom Sensor".to_string(), thermo_addr.clone())
                .expect("Failed to create remote thermometer");

        thread::sleep(Duration::from_millis(100));

        let thermo_config = ThermometerSimulatorConfig::new(
            "ThermoSim",
            thermo_addr.clone(),
            Duration::from_millis(100),
        )
        .with_pattern(TemperaturePattern::Constant(22.0));

        let _thermo_sim = simulator_spawner
            .spawn_thermometer_simulator("ThermoSim".to_string(), thermo_config)
            .unwrap();

        // Create remote outlet
        let remote_outlet = OutletRemote::new("Bedroom Lamp".to_string(), outlet_addr)
            .expect("Failed to create remote outlet");

        // Create room with both devices
        let room = create_room!(
            "Bedroom",
            "Bedroom Lamp" => Device::from(remote_outlet),
            "Bedroom Sensor" => Device::from(remote_thermo),
        );

        assert_eq!(room.devices.len(), 2);
        assert!(room.view_device("Bedroom Lamp").is_some());
        assert!(room.view_device("Bedroom Sensor").is_some());

        // Wait for thermometer update
        thread::sleep(Duration::from_millis(1500));

        // Verify both devices work
        let outlet_device = room.view_device("Bedroom Lamp").unwrap();
        match outlet_device {
            Device::OutletTypeRemote(outlet) => {
                assert_eq!(outlet.state().unwrap(), OutletState::Off);
            }
            _ => panic!("Expected OutletTypeRemote"),
        }

        let thermo_device = room.view_device("Bedroom Sensor").unwrap();
        match thermo_device {
            Device::ThermometerTypeRemote(thermo) => {
                let temp = thermo.current_temperature();
                assert!((temp - 22.0).abs() < 0.1, "Expected ~22.0, got {}", temp);
            }
            _ => panic!("Expected ThermometerTypeRemote"),
        }
    }

    #[test]
    fn smart_room_mixed_devices_test() {
        use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
        use crate::smart_devices::outlet_remote::OutletRemote;
        use std::thread;
        use std::time::Duration;

        // Spawn outlet simulator for remote device
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 200 as Watt);
        let simulator = OutletSimulator::spawn(config).expect("Failed to spawn simulator");
        let addr = simulator.address().to_string();

        thread::sleep(Duration::from_millis(100));

        let remote_outlet = OutletRemote::new("Remote Heater".to_string(), addr)
            .expect("Failed to create remote outlet");

        // Create room with both mock and remote devices
        let room = create_room!(
            "Office",
            "Local Lamp" => Device::new_outlet("Local Lamp".to_string(), OutletState::On, 60 as Watt),
            "Remote Heater" => Device::from(remote_outlet),
            "Local Thermometer" => Device::new_thermometer("Local Thermometer".to_string(), 21.0 as Celsius),
        );

        assert_eq!(room.devices.len(), 3);

        // Verify local device
        let local_outlet = room.view_device("Local Lamp").unwrap();
        match local_outlet {
            Device::OutletTypeMock(outlet) => {
                assert_eq!(outlet.state().unwrap(), OutletState::On);
                assert_eq!(outlet.power_usage().unwrap(), 60);
            }
            _ => panic!("Expected OutletTypeMock"),
        }

        // Verify remote device
        let remote_device = room.view_device("Remote Heater").unwrap();
        match remote_device {
            Device::OutletTypeRemote(outlet) => {
                assert_eq!(outlet.state().unwrap(), OutletState::Off);
            }
            _ => panic!("Expected OutletTypeRemote"),
        }

        // Verify local thermometer
        let local_thermo = room.view_device("Local Thermometer").unwrap();
        match local_thermo {
            Device::ThermometerTypeMock(thermo) => {
                assert_eq!(thermo.current_temperature(), 21.0);
            }
            _ => panic!("Expected ThermometerTypeMock"),
        }
    }

    #[test]
    fn smart_room_add_remove_remote_devices_test() {
        use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
        use crate::smart_devices::outlet_remote::OutletRemote;
        use std::thread;
        use std::time::Duration;

        let mut room = SmartRoom::new("Test Room".to_string(), HashMap::new());

        // Spawn simulators and create remote devices
        let config1 = OutletSimulatorConfig::new("127.0.0.1:0", 100 as Watt);
        let sim1 = OutletSimulator::spawn(config1).expect("Failed to spawn simulator 1");
        let addr1 = sim1.address().to_string();

        let config2 = OutletSimulatorConfig::new("127.0.0.1:0", 200 as Watt);
        let sim2 = OutletSimulator::spawn(config2).expect("Failed to spawn simulator 2");
        let addr2 = sim2.address().to_string();

        thread::sleep(Duration::from_millis(100));

        let remote1 =
            OutletRemote::new("Remote1".to_string(), addr1).expect("Failed to create remote1");
        let remote2 =
            OutletRemote::new("Remote2".to_string(), addr2).expect("Failed to create remote2");

        // Add devices
        room.add_device("Remote1".to_string(), Device::from(remote1));
        assert_eq!(room.devices.len(), 1);

        room.add_device("Remote2".to_string(), Device::from(remote2));
        assert_eq!(room.devices.len(), 2);

        // Verify both exist
        assert!(room.view_device("Remote1").is_some());
        assert!(room.view_device("Remote2").is_some());

        // Remove one device
        let removed = room.remove_device("Remote1");
        assert!(removed.is_some());
        assert_eq!(room.devices.len(), 1);
        assert!(room.view_device("Remote1").is_none());
        assert!(room.view_device("Remote2").is_some());

        // Remove second device
        let removed = room.remove_device("Remote2");
        assert!(removed.is_some());
        assert_eq!(room.devices.len(), 0);
    }

    #[test]
    fn smart_room_remote_device_info_test() {
        use crate::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};
        use crate::smart_devices::outlet_remote::OutletRemote;
        use std::thread;
        use std::time::Duration;

        let config = OutletSimulatorConfig::new("127.0.0.1:0", 175 as Watt);
        let simulator = OutletSimulator::spawn(config).expect("Failed to spawn simulator");
        let addr = simulator.address().to_string();

        thread::sleep(Duration::from_millis(100));

        let remote_outlet = OutletRemote::new("Smart Heater".to_string(), addr)
            .expect("Failed to create remote outlet");

        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        room.add_device("Smart Heater".to_string(), Device::from(remote_outlet));

        // Get device info
        let device = room.view_device("Smart Heater").unwrap();
        let info = device.info();

        // Should contain device name and status
        assert!(info.contains("Smart Heater"));
        assert!(info.contains("Remote Smart Outlet"));
    }
}
