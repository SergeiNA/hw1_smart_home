use crate::smart_devices::DeviceType;
use crate::traits::Information;

#[derive(Debug, Clone)]
pub struct SmartRoom {
    name: String,
    devices: Vec<DeviceType>,
}

impl Information for SmartRoom {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn info(&self) -> String {
        let enumerated_devices: Vec<String> = self
            .devices
            .iter()
            .enumerate()
            .map(|(i, d)| format!("[{}]: {}", i, d.info()))
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
    pub fn new(name: String, devices: Vec<DeviceType>) -> Self {
        SmartRoom { name, devices }
    }

    /// Returns an immutable reference to the device at the given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the device in the internal device list.
    ///
    /// # Panics
    ///
    /// This function will panic if `index` is out of bounds.
    pub fn view_device(&self, index: usize) -> &DeviceType {
        &self.devices[index]
    }

    /// Returns a mutable reference to the device at the given index,
    /// allowing the caller to modify the device.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the device in the internal device list.
    ///
    /// # Panics
    ///
    /// This function will panic if `index` is out of bounds.
    pub fn get_device(&mut self, index: usize) -> &mut DeviceType {
        &mut self.devices[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::{Celsius, OutletDevice, OutletState, Watt};

    #[test]
    fn smart_room_create_empty_test() {
        let room = SmartRoom::new("Living Room".to_string(), vec![]);
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn smart_room_view_index_out_of_bounds_test() {
        let room = SmartRoom::new("Living Room".to_string(), vec![]);
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
        let _ = room.view_device(0); // This should panic
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn smart_room_get_index_out_of_bounds_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), vec![]);
        assert_eq!(room.name(), "Living Room");
        assert_eq!(room.devices.len(), 0);
        let _ = room.get_device(0); // This should panic
    }

    #[test]
    fn smart_room_view_test() {
        let devices = vec![
            DeviceType::new_outlet("Lighter".to_string(), OutletState::On, 100 as Watt),
            DeviceType::new_outlet("PC".to_string(), OutletState::On, 250 as Watt),
            DeviceType::new_thermometer("Electronic thermometer".to_string(), 22.5 as Celsius),
        ];
        let room = SmartRoom::new("Living Room".to_string(), devices);

        assert_eq!(room.devices.len(), 3);
        assert_eq!(room.view_device(0).name(), "Lighter");
        assert_eq!(
            room.view_device(0).info(),
            "Smart Outlet: Lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(room.view_device(1).name(), "PC");
        assert_eq!(
            room.view_device(1).info(),
            "Smart Outlet: PC - Current State: On, Power Usage: 250 Watt"
        );
        assert_eq!(room.view_device(2).name(), "Electronic thermometer");
        assert_eq!(
            room.view_device(2).info(),
            "Thermometer: Electronic thermometer - Current Temperature: 22.50°C"
        );

        let outlet_lighter: &DeviceType = room.view_device(0);
        let outlet_pc: &DeviceType = room.view_device(1);
        let thermometer: &DeviceType = room.view_device(2);

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
        let mut room = SmartRoom::new("Living Room".to_string(), vec![]);
        let outlet =
            DeviceType::new_outlet("Smart Outlet".to_string(), OutletState::On, 150 as Watt);
        room.devices.push(outlet);
        assert_eq!(room.devices.len(), 1);
        assert_eq!(room.view_device(0).name(), "Smart Outlet");
    }

    #[test]
    fn smart_room_add_many_devices_test() {
        let mut room = SmartRoom::new("Living Room".to_string(), vec![]);
        room.devices.push(DeviceType::new_outlet(
            "Smart Outlet lighter".to_string(),
            OutletState::On,
            100 as Watt,
        ));
        room.devices.push(DeviceType::new_outlet(
            "Smart Outlet PC".to_string(),
            OutletState::On,
            250 as Watt,
        ));
        room.devices.push(DeviceType::new_thermometer(
            "Smart Thermometer".to_string(),
            22.5 as Celsius,
        ));
        assert_eq!(room.devices.len(), 3);
        assert_eq!(room.view_device(0).name(), "Smart Outlet lighter");
        assert_eq!(room.view_device(1).name(), "Smart Outlet PC");
        assert_eq!(room.view_device(2).name(), "Smart Thermometer");

        let expected = r#"
Smart Room: Living Room:
 Total devices: 3
  [0]: Smart Outlet: Smart Outlet lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [2]: Thermometer: Smart Thermometer - Current Temperature: 22.50°C"#;
        assert_eq!(room.info(), expected);
    }

    #[test]
    fn smart_room_from_vec_create_test() {
        let devices = vec![
            DeviceType::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            DeviceType::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
            DeviceType::new_thermometer("Smart Thermometer".to_string(), 22.5 as Celsius),
        ];
        let room = SmartRoom::new("Living Room".to_string(), devices);

        assert_eq!(room.devices.len(), 3);
        assert_eq!(room.view_device(0).name(), "Smart Outlet lighter");
        assert_eq!(room.view_device(1).name(), "Smart Outlet PC");
        assert_eq!(room.view_device(2).name(), "Smart Thermometer");

        let expected = r#"
Smart Room: Living Room:
 Total devices: 3
  [0]: Smart Outlet: Smart Outlet lighter - Current State: On, Power Usage: 100 Watt
  --------------------------------------
  [1]: Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt
  --------------------------------------
  [2]: Thermometer: Smart Thermometer - Current Temperature: 22.50°C"#;
        assert_eq!(room.info(), expected);
    }

    #[test]
    fn smart_room_ref_device_test() {
        let devices = vec![
            DeviceType::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            DeviceType::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        ];
        let room = SmartRoom::new("Living Room".to_string(), devices);

        let device_0 = room.view_device(0);
        let outlet_0 = match device_0 {
            DeviceType::OutletType(o) => o,
            _ => panic!("Expected OutletType"),
        };

        assert_eq!(outlet_0.name(), "Smart Outlet lighter");
        assert_eq!(outlet_0.state(), OutletState::On);
        assert_eq!(outlet_0.power_usage(), 100 as Watt);
    }

    #[test]
    fn smart_room_get_test() {
        let devices = vec![
            DeviceType::new_outlet(
                "Smart Outlet lighter".to_string(),
                OutletState::On,
                100 as Watt,
            ),
            DeviceType::new_outlet("Smart Outlet PC".to_string(), OutletState::On, 250 as Watt),
        ];
        let mut room = SmartRoom::new("Living Room".to_string(), devices);

        assert_eq!(room.devices.len(), 2);
        assert_eq!(
            room.view_device(0).info(),
            "Smart Outlet: Smart Outlet lighter - Current State: On, Power Usage: 100 Watt"
        );
        assert_eq!(
            room.view_device(1).info(),
            "Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt"
        );

        {
            let outlet_device: &mut DeviceType = room.get_device(0);
            let outlet = match outlet_device {
                DeviceType::OutletType(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::Off);
        }
        assert_eq!(
            room.get_device(0).info(),
            "Smart Outlet: Smart Outlet lighter - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            room.view_device(1).info(),
            "Smart Outlet: Smart Outlet PC - Current State: On, Power Usage: 250 Watt"
        );

        {
            let outlet_device: &mut DeviceType = room.get_device(1);
            let outlet = match outlet_device {
                DeviceType::OutletType(o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
            outlet.switch();
            assert_eq!(outlet.state(), OutletState::Off);
        }
        assert_eq!(
            room.view_device(0).info(),
            "Smart Outlet: Smart Outlet lighter - Current State: Off, Power Usage: 0 Watt"
        );
        assert_eq!(
            room.view_device(1).info(),
            "Smart Outlet: Smart Outlet PC - Current State: Off, Power Usage: 0 Watt"
        );
    }
}
