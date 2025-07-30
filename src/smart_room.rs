use crate::smart_devices::Device;
use crate::traits::Information;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::Display;
use std::string::String;

#[derive(Debug, Clone)]
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
        let sorted_devices: BTreeMap<String, Device> = self.clone().devices.into_iter().collect();
        let enumerated_devices: Vec<String> = sorted_devices
            .iter()
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
                key, self.name
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
    use crate::smart_devices::{Celsius, OutletDevice, OutletState, Watt};

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
        assert_eq!(room.view_device("Some device"), None);
    }

    #[test]
    fn smart_room_get_index_out_of_bounds_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
        assert_eq!(room.get_device("Some device"), None);
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
        assert_eq!(room.view_device("Some device"), None);
        assert_eq!(room.view_device("Lighter").is_some(), true);
        assert_eq!(
            room.view_device("Lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
                .name(),
            "Lighter"
        );
        assert_eq!(
            room.view_device("Lighter")
                .unwrap_or(&TEST_DEFAULT_DEVICE)
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

        assert_eq!(room.remove_device("Not existing device"), None);
        assert_eq!(room.devices.len(), 1);

        let removed_device = room
            .remove_device("Smart Outlet")
            .unwrap_or(TEST_DEFAULT_DEVICE);
        assert_eq!(removed_device.name(), "Smart Outlet");
        assert_eq!(room.devices.len(), 0);

        assert_eq!(room.remove_device("Smart Outlet"), None);
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

        assert_eq!(room.remove_device("Smart Outlet lighter"), None);
        assert_eq!(room.remove_device("Smart Outlet PC"), None);
        assert_eq!(room.remove_device("Smart Thermometer"), None);
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
            Device::OutletType(o) => o,
            _ => panic!("Expected OutletType"),
        };

        assert_eq!(outlet_0.name(), "Smart Outlet lighter");
        assert_eq!(outlet_0.state(), OutletState::On);
        assert_eq!(outlet_0.power_usage(), 100 as Watt);
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
                Device::OutletType(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::Off);
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
                Device::OutletType(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::Off);
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
}
