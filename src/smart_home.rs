use crate::smart_devices::Device;
use crate::smart_room::{AccessDevice, SmartRoom};
use crate::traits::Information;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct SmartHome {
    name: String,
    rooms: HashMap<String, SmartRoom>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoomAccessError {
    pub message: String,
}

impl Display for RoomAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoomAccessError: {}", self.message)
    }
}

impl Error for RoomAccessError {}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceAccessError {
    DeviceAccess(crate::smart_room::AccessError),
    RoomAccess(RoomAccessError),
}

impl Display for DeviceAccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceAccessError::DeviceAccess(e) => write!(f, "Error: {e}"),
            DeviceAccessError::RoomAccess(e) => write!(f, "Error: {e}"),
        }
    }
}

impl From<crate::smart_room::AccessError> for DeviceAccessError {
    fn from(error: crate::smart_room::AccessError) -> Self {
        DeviceAccessError::DeviceAccess(error)
    }
}

impl From<RoomAccessError> for DeviceAccessError {
    fn from(error: RoomAccessError) -> Self {
        DeviceAccessError::RoomAccess(error)
    }
}

impl Error for DeviceAccessError {}

impl Information for SmartHome {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn info(&self) -> String {
        let sorted_rooms: BTreeMap<String, SmartRoom> = self.clone().rooms.into_iter().collect();
        let enumerated_rooms: Vec<String> = sorted_rooms
            .iter()
            .enumerate()
            .map(|(i, r)| format!("Room[{}]:{}", i, r.1.info()))
            .collect();
        format!(
            "Smart Home: {}:\n Total Rooms: {}\n\n{}",
            self.name,
            enumerated_rooms.len(),
            enumerated_rooms.join("\n=====================================\n")
        )
    }
}

impl SmartHome {
    /// Creates a new SmartHome with the given name and rooms
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the smart home
    /// * `rooms` - A HashMap of room names to SmartRoom instances
    pub fn new(name: String, rooms: HashMap<String, SmartRoom>) -> Self {
        SmartHome { name, rooms }
    }

    /// Returns an immutable reference to the room with the specified name.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the room to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the SmartRoom if found, None otherwise.
    pub fn view_room(&self, key: &str) -> Option<&SmartRoom> {
        self.rooms.get(key)
    }

    /// Returns a mutable reference to the room with the specified name,
    /// allowing the caller to modify the room.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the room to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing a mutable reference to the SmartRoom if found, None otherwise.
    pub fn get_room(&mut self, key: &str) -> Option<&mut SmartRoom> {
        self.rooms.get_mut(key)
    }

    /// Attempts to access a device in a specific room
    ///
    /// # Arguments
    ///
    /// * `room_name` - The name of the room containing the device
    /// * `device_name` - The name of the device to access
    ///
    /// # Returns
    ///
    /// A Result containing either a reference to the device if found, or a DeviceAccessError
    pub fn device(&self, room_name: &str, device_name: &str) -> Result<&Device, DeviceAccessError> {
        Ok(self.access_room(room_name)?.access_device(device_name)?)
    }

    /// Adds a room to the smart home
    ///
    /// # Arguments
    ///
    /// * `room` - The SmartRoom to add
    pub fn add_room(&mut self, room: SmartRoom) {
        self.rooms.insert(room.name().clone(), room);
    }

    /// Removes a room from the smart home by name
    ///
    /// # Arguments
    ///
    /// * `room` - The name of the room to remove
    ///
    /// # Returns
    ///
    /// An Option containing the removed SmartRoom if it existed, None otherwise
    pub fn remove_room(&mut self, room: &str) -> Option<SmartRoom> {
        self.rooms.remove(room)
    }
}

/// A trait for accessing rooms in a smart home system
///
/// This trait defines the behavior for accessing rooms in a container (like SmartHome)
/// and provides error handling for when a room cannot be found.
pub trait AccessRoom {
    /// Attempts to access a room by its name
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the room to access
    ///
    /// # Returns
    ///
    /// A Result containing either a reference to the SmartRoom if found,
    /// or a RoomAccessError if the room does not exist
    fn access_room(&self, key: &str) -> Result<&SmartRoom, RoomAccessError>;
}

impl AccessRoom for SmartHome {
    /// Implementation of `access_room` for SmartHome
    ///
    /// Attempts to retrieve a room by name and returns a Result.
    /// If the room is not found, returns a RoomAccessError with a descriptive message.
    fn access_room(&self, key: &str) -> Result<&SmartRoom, RoomAccessError> {
        self.view_room(key).ok_or(RoomAccessError {
            message: format!(
                "Room with the name '{}' not found in the house '{}'",
                key, self.name
            ),
        })
    }
}

#[macro_export]
macro_rules! create_home {
    ($name:expr, $({ $key:expr , $value:expr }),* $(,)? ) => {{
        let rooms = std::collections::HashMap::from(
          [
            $( ($key.to_string(), $value) ),*
          ]
        );
        SmartHome::new($name.to_string(), rooms)
    }};
}

#[cfg(test)]
mod tests {
    use crate::create_room;
    use crate::smart_devices::{Celsius, Device, OutletDevice, OutletState, Watt};
    use crate::smart_home::{AccessRoom, DeviceAccessError, RoomAccessError, SmartHome};
    use crate::smart_room::SmartRoom;
    use crate::traits::Information;
    use std::collections::HashMap;

    #[test]
    fn view_home_rooms() {
        let room1 = SmartRoom::new("Living Room".to_string(), HashMap::new());
        let room2 = SmartRoom::new("Bedroom".to_string(), HashMap::new());
        let home = SmartHome::new(
            "My Home".to_string(),
            HashMap::from([
                ("Living Room".to_string(), room1),
                ("Bedroom".to_string(), room2),
            ]),
        );

        assert_eq!(home.view_room("Living Room").unwrap().name(), "Living Room");
        assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");
    }

    #[test]
    fn create_home_test() {
        {
            let home = create_home!(
                "My Smart Home",
                {"Living Room",
                SmartRoom::new("Living Room".to_string(), HashMap::new())},
                {"Bedroom",
                SmartRoom::new("Bedroom".to_string(), HashMap::new())},

            );

            assert_eq!(home.name(), "My Smart Home");
            assert_eq!(home.view_room("Living Room").unwrap().name(), "Living Room");
            assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");
        }
        {
            let home = create_home!(
                "My Smart Home",
                {
                    "Bedroom",
                    create_room!(
                        "Bedroom",
                        "Attached Outlet" => Device::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
                        "Light Outlet" => Device::new_outlet("Light Outlet".to_string(), OutletState::Off, 150 as Watt),
                        "Electron thermometer" => Device::new_thermometer("Electron thermometer".to_string(), 22.5 as Celsius))
                },
            );

            assert_eq!(home.name(), "My Smart Home");
            assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");
            assert!(home.view_room("Living Room").is_none());
        }
    }

    #[test]
    fn view_home_devices() {
        let bedroom = create_room!(
            "Bedroom",
            "Attached Outlet" => Device::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
            "Light Outlet" => Device::new_outlet("Light Outlet".to_string(), OutletState::Off, 150 as Watt),
            "Electron thermometer" => Device::new_thermometer("Electron thermometer".to_string(), 22.5 as Celsius)
        );

        let home = SmartHome::new(
            "My Smart Home".to_string(),
            HashMap::from([("Bedroom".to_string(), bedroom)]),
        );

        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Attached Outlet")
                .unwrap()
                .name(),
            "Attached Outlet"
        );
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Attached Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt"
        );
    }

    #[test]
    fn smart_home_base_test() {
        let home = create_home!(
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

        assert_eq!(home.name(), "My Smart Home");
        assert_eq!(home.rooms.len(), 3);
        assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");
        assert_eq!(home.view_room("Living Room").unwrap().name(), "Living Room");
        assert_eq!(
            home.view_room("Kitchen Room").unwrap().name(),
            "Kitchen Room"
        );

        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Attached Outlet")
                .unwrap()
                .name(),
            "Attached Outlet"
        );
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Attached Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Light Outlet")
                .unwrap()
                .name(),
            "Light Outlet"
        );
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Light Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Electron thermometer")
                .unwrap()
                .name(),
            "Electron thermometer"
        );
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Electron thermometer")
                .unwrap()
                .info(),
            "Thermometer: Electron thermometer - Current Temperature: 22.50°C"
        );

        assert_eq!(
            home.view_room("Living Room")
                .unwrap()
                .view_device("Lighter")
                .unwrap()
                .name(),
            "Lighter"
        );
        assert_eq!(
            home.view_room("Living Room")
                .unwrap()
                .view_device("Lighter")
                .unwrap()
                .info(),
            "Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(
            home.view_room("Living Room")
                .unwrap()
                .view_device("PC")
                .unwrap()
                .name(),
            "PC"
        );
        assert_eq!(
            home.view_room("Living Room")
                .unwrap()
                .view_device("PC")
                .unwrap()
                .info(),
            "Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(
            home.view_room("Living Room")
                .unwrap()
                .view_device("Electronic thermometer")
                .unwrap()
                .name(),
            "Electronic thermometer"
        );
        assert_eq!(
            home.view_room("Living Room")
                .unwrap()
                .view_device("Electronic thermometer")
                .unwrap()
                .info(),
            "Thermometer: Electronic thermometer - Current Temperature: 22.50°C"
        );

        assert_eq!(
            home.view_room("Kitchen Room")
                .unwrap()
                .view_device("Refrigerator Outlet")
                .unwrap()
                .name(),
            "Refrigerator Outlet"
        );
        assert_eq!(
            home.view_room("Kitchen Room")
                .unwrap()
                .view_device("Refrigerator Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(
            home.view_room("Kitchen Room")
                .unwrap()
                .view_device("Teapot Outlet")
                .unwrap()
                .name(),
            "Teapot Outlet"
        );
        assert_eq!(
            home.view_room("Kitchen Room")
                .unwrap()
                .view_device("Teapot Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Teapot Outlet - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            home.view_room("Kitchen Room")
                .unwrap()
                .view_device("Kitchen thermometer")
                .unwrap()
                .name(),
            "Kitchen thermometer"
        );
        assert_eq!(
            home.view_room("Kitchen Room")
                .unwrap()
                .view_device("Kitchen thermometer")
                .unwrap()
                .info(),
            "Thermometer: Kitchen thermometer - Current Temperature: 20.00°C"
        );

        let expected = r#"Smart Home: My Smart Home:
 Total Rooms: 3

Room[0]:
Smart Room: Bedroom:
 Total devices: 3
  [0]: Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Thermometer: Electron thermometer - Current Temperature: 22.50°C
  --------------------------------------
  [2]: Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt
=====================================
Room[1]:
Smart Room: Kitchen Room:
 Total devices: 3
  [0]: Thermometer: Kitchen thermometer - Current Temperature: 20.00°C
  --------------------------------------
  [1]: Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Smart Outlet: Teapot Outlet - Current State: Off, Power Usage: 0 Watt
=====================================
Room[2]:
Smart Room: Living Room:
 Total devices: 3
  [0]: Thermometer: Electronic thermometer - Current Temperature: 22.50°C
  --------------------------------------
  [1]: Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"#;

        assert_eq!(home.info(), expected);
    }

    #[test]
    fn smart_home_add_rooms_test() {
        let mut home = SmartHome::new("My Home".to_string(), HashMap::new());
        let bedroom = SmartRoom::new("Bedroom".to_string(), HashMap::new());
        home.add_room(bedroom);
        assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");

        let living_room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        home.add_room(living_room);
        assert_eq!(home.view_room("Living Room").unwrap().name(), "Living Room");

        assert_eq!(home.rooms.len(), 2);
    }

    #[test]
    fn smart_home_remove_rooms_test() {
        let mut home = SmartHome::new("My Home".to_string(), HashMap::new());
        let bedroom = SmartRoom::new("Bedroom".to_string(), HashMap::new());
        home.add_room(bedroom);
        assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");

        home.remove_room("Bedroom");
        assert!(home.view_room("Bedroom").is_none());

        assert_eq!(home.rooms.len(), 0);
    }

    #[test]
    fn smart_home_access_room_test() {
        let bedroom = SmartRoom::new("Bedroom".to_string(), HashMap::new());
        let living_room = SmartRoom::new("Living Room".to_string(), HashMap::new());
        let home = SmartHome::new(
            "My Home".to_string(),
            HashMap::from([
                ("Bedroom".to_string(), bedroom),
                ("Living Room".to_string(), living_room),
            ]),
        );

        assert!(home.access_room("Bedroom").is_ok());
        assert!(home.access_room("Living Room").is_ok());
        assert!(home.access_room("Kitchen").is_err());

        assert_eq!(
            home.access_room("Kitchen").unwrap_err(),
            RoomAccessError {
                message: "Room with the name 'Kitchen' not found in the house 'My Home'"
                    .to_string()
            }
        );
    }

    #[test]
    fn smart_home_access_device_test() {
        let bedroom = SmartRoom::new(
            "Bedroom".to_string(),
            HashMap::from([(
                "Attached Outlet".to_string(),
                Device::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
            )]),
        );
        let home = SmartHome::new(
            "My Home".to_string(),
            HashMap::from([("Bedroom".to_string(), bedroom)]),
        );

        assert!(home.device("Bedroom", "Attached Outlet").is_ok());
        assert!(home.device("Bedroom", "Nonexistent Device").is_err());
        assert_eq!(
            home.device("Bedroom", "Nonexistent Device").unwrap_err(),
            DeviceAccessError::DeviceAccess(crate::smart_room::AccessError {
                message:
                    "Device with the name 'Nonexistent Device' not found in the room 'Bedroom'"
                        .to_string()
            })
        );
    }

    #[test]
    fn smart_room_get_ne_exist_room_test() {
        let home = SmartHome::new("My Home".to_string(), HashMap::new());
        let none = home.view_room("Some room");
        assert!(none.is_none());
    }

    #[test]
    fn smart_home_switch_outlet_test() {
        let bedroom = create_room!(
            "Bedroom",
            "Attached Outlet" => Device::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
            "Light Outlet" => Device::new_outlet("Light Outlet".to_string(), OutletState::Off, 150 as Watt),
            "Electron thermometer" => Device::new_thermometer("Electron thermometer".to_string(), 22.5 as Celsius)
        );
        let living_room = create_room!(
            "Living Room",
            "Lighter" => Device::new_outlet("Lighter".to_string(), OutletState::On, 100 as Watt),
            "PC" => Device::new_outlet("PC".to_string(), OutletState::On, 250 as Watt),
            "Electronic thermometer" => Device::new_thermometer("Electronic thermometer".to_string(), 22.5 as Celsius)
        );
        let mut home = create_home!(
            "My Home",
            {"Bedroom", bedroom},
            {"Living Room", living_room}
        );

        assert_eq!(home.view_room("Bedroom").unwrap().name(), "Bedroom");
        assert_eq!(home.view_room("Living Room").unwrap().name(), "Living Room");

        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Attached Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt"
        );

        {
            let device = home
                .get_room("Bedroom")
                .unwrap()
                .get_device("Attached Outlet")
                .unwrap();
            let outlet = match device {
                Device::OutletType(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::Off);
        }
        assert_eq!(
            home.view_room("Bedroom")
                .unwrap()
                .view_device("Attached Outlet")
                .unwrap()
                .info(),
            "Smart Outlet: Attached Outlet - Current State: Off, Power Usage: 0 Watt"
        );
    }
    #[test]
    fn smart_home_changed_info_test() {
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

        let expected = r#"Smart Home: My Smart Home:
 Total Rooms: 3

Room[0]:
Smart Room: Bedroom:
 Total devices: 3
  [0]: Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Thermometer: Electron thermometer - Current Temperature: 22.50°C
  --------------------------------------
  [2]: Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt
=====================================
Room[1]:
Smart Room: Kitchen Room:
 Total devices: 3
  [0]: Thermometer: Kitchen thermometer - Current Temperature: 20.00°C
  --------------------------------------
  [1]: Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Smart Outlet: Teapot Outlet - Current State: Off, Power Usage: 0 Watt
=====================================
Room[2]:
Smart Room: Living Room:
 Total devices: 3
  [0]: Thermometer: Electronic thermometer - Current Temperature: 22.50°C
  --------------------------------------
  [1]: Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"#;

        assert_eq!(home.info(), expected);

        {
            let kitchen_room = home.get_room("Kitchen Room").unwrap();
            let device = kitchen_room.get_device("Teapot Outlet").unwrap();
            let outlet = match device {
                Device::OutletType(outlet) => outlet,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::Off);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::On);
        }

        let expected = r#"Smart Home: My Smart Home:
 Total Rooms: 3

Room[0]:
Smart Room: Bedroom:
 Total devices: 3
  [0]: Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Thermometer: Electron thermometer - Current Temperature: 22.50°C
  --------------------------------------
  [2]: Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt
=====================================
Room[1]:
Smart Room: Kitchen Room:
 Total devices: 3
  [0]: Thermometer: Kitchen thermometer - Current Temperature: 20.00°C
  --------------------------------------
  [1]: Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Smart Outlet: Teapot Outlet - Current State: On, Power Usage: 150 Watt
=====================================
Room[2]:
Smart Room: Living Room:
 Total devices: 3
  [0]: Thermometer: Electronic thermometer - Current Temperature: 22.50°C
  --------------------------------------
  [1]: Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [2]: Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"#;

        assert_eq!(home.info(), expected);
    }
}
