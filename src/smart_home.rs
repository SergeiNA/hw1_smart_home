use crate::smart_room::SmartRoom;
use crate::traits::Information;

pub struct SmartHome {
    name: String,
    rooms: Vec<SmartRoom>,
}

impl Information for SmartHome {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn info(&self) -> String {
        let enumerated_rooms: Vec<String> = self
            .rooms
            .iter()
            .enumerate()
            .map(|(i, r)| format!("Room[{}]:{}", i, r.info()))
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
    pub fn new(name: String, rooms: Vec<SmartRoom>) -> Self {
        SmartHome { name, rooms }
    }

    /// Returns an immutable reference to the room at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the room in the internal `rooms` list.
    ///
    /// # Panics
    ///
    /// This function will panic if the index is out of bounds.
    pub fn view_room(&self, index: usize) -> &SmartRoom {
        &self.rooms[index]
    }

    /// Returns a mutable reference to the room at the specified index,
    /// allowing the caller to modify the room.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the room in the internal `rooms` list.
    ///
    /// # Panics
    ///
    /// This function will panic if the index is out of bounds.
    pub fn get_room(&mut self, index: usize) -> &mut SmartRoom {
        &mut self.rooms[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::smart_devices::{Celsius, DeviceType, OutletDevice, OutletState, Watt};
    use crate::smart_home::SmartHome;
    use crate::smart_room::SmartRoom;
    use crate::traits::Information;

    #[test]
    fn view_home_rooms() {
        let room1 = SmartRoom::new("Living Room".to_string(), vec![]);
        let room2 = SmartRoom::new("Bedroom".to_string(), vec![]);
        let home = SmartHome::new("My Home".to_string(), vec![room1, room2]);

        assert_eq!(home.view_room(0).name(), "Living Room");
        assert_eq!(home.view_room(1).name(), "Bedroom");
    }

    #[test]
    fn view_home_devices() {
        let bedroom = SmartRoom::new(
            "Bedroom".to_string(),
            vec![
                DeviceType::new_outlet("Attached Outlet".to_string(), OutletState::On, 250 as Watt),
                DeviceType::new_outlet("Light Outlet".to_string(), OutletState::Off, 150 as Watt),
                DeviceType::new_thermometer("Electron thermometer".to_string(), 22.5 as Celsius),
            ],
        );

        let home = SmartHome::new("My Smart Home".to_string(), vec![bedroom]);

        assert_eq!(home.view_room(0).view_device(0).name(), "Attached Outlet");
        assert_eq!(
            home.view_room(0).view_device(0).info(),
            "Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt"
        );
    }

    #[test]
    fn smart_home_base_test() {
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

        let home = SmartHome::new(
            "My Smart Home".to_string(),
            vec![bedroom, living_room, kitchen],
        );

        assert_eq!(home.name(), "My Smart Home");
        assert_eq!(home.rooms.len(), 3);
        assert_eq!(home.view_room(0).name(), "Bedroom");
        assert_eq!(home.view_room(1).name(), "Living Room");
        assert_eq!(home.view_room(2).name(), "Kitchen Room");

        assert_eq!(home.view_room(0).view_device(0).name(), "Attached Outlet");
        assert_eq!(
            home.view_room(0).view_device(0).info(),
            "Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(home.view_room(0).view_device(1).name(), "Light Outlet");
        assert_eq!(
            home.view_room(0).view_device(1).info(),
            "Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            home.view_room(0).view_device(2).name(),
            "Electron thermometer"
        );
        assert_eq!(
            home.view_room(0).view_device(2).info(),
            "Thermometer: Electron thermometer - Current Temperature: 22.50°C"
        );

        assert_eq!(home.view_room(1).view_device(0).name(), "Lighter");
        assert_eq!(
            home.view_room(1).view_device(0).info(),
            "Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(home.view_room(1).view_device(1).name(), "PC");
        assert_eq!(
            home.view_room(1).view_device(1).info(),
            "Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(
            home.view_room(1).view_device(2).name(),
            "Electronic thermometer"
        );
        assert_eq!(
            home.view_room(1).view_device(2).info(),
            "Thermometer: Electronic thermometer - Current Temperature: 22.50°C"
        );

        assert_eq!(
            home.view_room(2).view_device(0).name(),
            "Refrigerator Outlet"
        );
        assert_eq!(
            home.view_room(2).view_device(0).info(),
            "Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(home.view_room(2).view_device(1).name(), "Teapot Outlet");
        assert_eq!(
            home.view_room(2).view_device(1).info(),
            "Smart Outlet: Teapot Outlet - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            home.view_room(2).view_device(2).name(),
            "Kitchen thermometer"
        );
        assert_eq!(
            home.view_room(2).view_device(2).info(),
            "Thermometer: Kitchen thermometer - Current Temperature: 20.00°C"
        );

        let expected = r#"Smart Home: My Smart Home:
 Total Rooms: 3

Room[0]:
Smart Room: Bedroom:
 Total devices: 3
  [0]: Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt
  --------------------------------------
  [2]: Thermometer: Electron thermometer - Current Temperature: 22.50°C
=====================================
Room[1]:
Smart Room: Living Room:
 Total devices: 3
  [0]: Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: PC - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [2]: Thermometer: Electronic thermometer - Current Temperature: 22.50°C
=====================================
Room[2]:
Smart Room: Kitchen Room:
 Total devices: 3
  [0]: Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: Teapot Outlet - Current State: Off, Power Usage: 0 Watt
  --------------------------------------
  [2]: Thermometer: Kitchen thermometer - Current Temperature: 20.00°C"#;

        assert_eq!(home.info(), expected);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn smart_room_get_index_out_of_bounds_test() {
        let home = SmartHome::new("My Home".to_string(), vec![]);
        let _ = home.view_room(0).name();
    }

    #[test]
    fn smart_home_switch_outlet_test() {
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
        let mut home = SmartHome::new("My Home".to_string(), vec![bedroom, living_room]);

        assert_eq!(home.view_room(0).name(), "Bedroom");
        assert_eq!(home.view_room(1).name(), "Living Room");

        assert_eq!(
            home.view_room(0).view_device(0).info(),
            "Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt"
        );

        {
            let device = home.get_room(0).get_device(0);
            let outlet = match device {
                DeviceType::OutletType(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::Off);
        }
        assert_eq!(
            home.view_room(0).view_device(0).info(),
            "Smart Outlet: Attached Outlet - Current State: Off, Power Usage: 0 Watt"
        );
    }
    #[test]
    fn smart_home_changed_info_test() {
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

        let expected = r#"Smart Home: My Smart Home:
 Total Rooms: 3

Room[0]:
Smart Room: Bedroom:
 Total devices: 3
  [0]: Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt
  --------------------------------------
  [2]: Thermometer: Electron thermometer - Current Temperature: 22.50°C
=====================================
Room[1]:
Smart Room: Living Room:
 Total devices: 3
  [0]: Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: PC - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [2]: Thermometer: Electronic thermometer - Current Temperature: 22.50°C
=====================================
Room[2]:
Smart Room: Kitchen Room:
 Total devices: 3
  [0]: Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: Teapot Outlet - Current State: Off, Power Usage: 0 Watt
  --------------------------------------
  [2]: Thermometer: Kitchen thermometer - Current Temperature: 20.00°C"#;

        assert_eq!(home.info(), expected);

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

        let expected = r#"Smart Home: My Smart Home:
 Total Rooms: 3

Room[0]:
Smart Room: Bedroom:
 Total devices: 3
  [0]: Smart Outlet: Attached Outlet - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [1]: Smart Outlet: Light Outlet - Current State: Off, Power Usage: 0 Watt
  --------------------------------------
  [2]: Thermometer: Electron thermometer - Current Temperature: 22.50°C
=====================================
Room[1]:
Smart Room: Living Room:
 Total devices: 3
  [0]: Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: PC - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [2]: Thermometer: Electronic thermometer - Current Temperature: 22.50°C
=====================================
Room[2]:
Smart Room: Kitchen Room:
 Total devices: 3
  [0]: Smart Outlet: Refrigerator Outlet - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: Teapot Outlet - Current State: On, Power Usage: 150 Watt
  --------------------------------------
  [2]: Thermometer: Kitchen thermometer - Current Temperature: 20.00°C"#;

        assert_eq!(home.info(), expected);
    }
}
