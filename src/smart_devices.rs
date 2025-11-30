pub mod errors;
pub mod outlet_mock;
pub mod outlet_remote;
pub mod thermometer_mock;
pub mod thermometer_remote;
pub mod types;

use crate::smart_devices::outlet_remote::OutletRemote;
use crate::smart_devices::thermometer_remote::ThermometerRemote;
use crate::smart_devices::types::OutletState;
use crate::traits::Information;
pub use outlet_mock::{OutletDevice, OutletMock};
pub use thermometer_mock::{TemperatureSensor, ThermometerMock};
pub use types::{Celsius, Fahrenheit, Kelvin, Watt};

#[derive(Debug)]
pub enum Device {
    OutletTypeMock(OutletMock),
    OutletTypeRemote(OutletRemote),
    ThermometerTypeMock(ThermometerMock),
    ThermometerTypeRemote(ThermometerRemote),
    Empty,
}

impl From<OutletMock> for Device {
    fn from(outlet: OutletMock) -> Self {
        Device::OutletTypeMock(outlet)
    }
}

impl From<OutletRemote> for Device {
    fn from(outlet: OutletRemote) -> Self {
        Device::OutletTypeRemote(outlet)
    }
}

impl From<ThermometerMock> for Device {
    fn from(thermometer: ThermometerMock) -> Self {
        Device::ThermometerTypeMock(thermometer)
    }
}

impl From<ThermometerRemote> for Device {
    fn from(thermometer: ThermometerRemote) -> Self {
        Device::ThermometerTypeRemote(thermometer)
    }
}

impl Information for Device {
    fn name(&self) -> String {
        match self {
            Device::OutletTypeMock(outlet) => outlet.name(),
            Device::OutletTypeRemote(outlet) => outlet.name(),
            Device::ThermometerTypeMock(thermometer) => thermometer.name(),
            Device::ThermometerTypeRemote(thermometer) => thermometer.name(),
            Device::Empty => "No Device".to_string(),
        }
    }

    fn info(&self) -> String {
        match self {
            Device::OutletTypeMock(outlet) => outlet.info(),
            Device::OutletTypeRemote(outlet) => outlet.info(),
            Device::ThermometerTypeMock(thermometer) => thermometer.info(),
            Device::ThermometerTypeRemote(thermometer) => thermometer.info(),
            Device::Empty => "No device information available".to_string(),
        }
    }
}

impl Device {
    pub fn new_outlet(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        Device::OutletTypeMock(OutletMock::new(name, initial_state, power_usage))
    }

    pub fn new_thermometer(name: String, initial_temperature: Celsius) -> Self {
        Device::ThermometerTypeMock(ThermometerMock::new(name, initial_temperature))
    }

    // TODO: Add more constructors for Remote types when needed
    // Don't have time, use from implementations for now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_outlet_create_test() {
        let outlet = Device::new_outlet("Living Room".to_string(), OutletState::On, 150);
        assert_eq!(outlet.name(), "Living Room");
        assert_eq!(
            outlet.info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
    }

    #[test]
    fn device_type_thermometer_create_test() {
        let thermometer = Device::new_thermometer("Bedroom".to_string(), 22.5 as Celsius);
        assert_eq!(thermometer.name(), "Bedroom");
        assert_eq!(
            thermometer.info(),
            "Thermometer: Bedroom - Current Temperature: 22.50°C"
        );
    }

    #[test]
    fn device_type_thermometer_get_test() {
        let thermometer = Device::new_thermometer("Bedroom".to_string(), 22.5 as Celsius);
        assert_eq!(thermometer.name(), "Bedroom");
        assert_eq!(
            thermometer.info(),
            "Thermometer: Bedroom - Current Temperature: 22.50°C"
        );
        {
            let t: &ThermometerMock = match thermometer {
                Device::ThermometerTypeMock(ref t) => t,
                _ => panic!("Expected ThermometerType"),
            };
            assert_eq!(t.current_temperature(), 22.5 as Celsius);
        }
    }

    #[test]
    fn device_type_outlet_switch_test() {
        let mut outlet_device = Device::new_outlet("Living Room".to_string(), OutletState::On, 150);
        assert_eq!(outlet_device.name(), "Living Room");
        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
        {
            let outlet: &mut OutletMock = match outlet_device {
                Device::OutletTypeMock(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.switch().expect("Failed to switch outlet state");
            assert_eq!(outlet.state().unwrap(), OutletState::Off);
        }

        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: Off, Power Usage: 0 Watt"
        );
    }

    #[test]
    fn device_type_outlet_turn_on_off_test() {
        let mut outlet_device = Device::new_outlet("Living Room".to_string(), OutletState::On, 150);
        assert_eq!(outlet_device.name(), "Living Room");
        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
        {
            let outlet: &mut OutletMock = match outlet_device {
                Device::OutletTypeMock(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.turn_off().expect("Failed to turn off outlet state");
        }
        {
            let outlet: &mut OutletMock = match outlet_device {
                Device::OutletTypeMock(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(
                outlet.state().expect("Failed to get outlet state"),
                OutletState::Off
            );
        }
        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: Off, Power Usage: 0 Watt"
        );
        {
            let outlet: &mut OutletMock = match outlet_device {
                Device::OutletTypeMock(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.turn_on().expect("Failed to turn on outlet state");
        }
        {
            let outlet: &mut OutletMock = match outlet_device {
                Device::OutletTypeMock(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(
                outlet.state().expect("Failed to get state"),
                OutletState::On
            );
        }
        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
    }

    #[test]
    fn device_from_test() {
        let outlet = OutletMock::new("Test Outlet".to_string(), OutletState::On, 200);
        let device_from_outlet: Device = outlet.into();
        assert_eq!(device_from_outlet.name(), "Test Outlet");
        assert_eq!(
            device_from_outlet.info(),
            "Smart Outlet: Test Outlet - Current State: On, Power Usage: 200 Watt"
        );

        let thermometer = ThermometerMock::new("Test Thermometer".to_string(), 25.0 as Celsius);
        let device_from_thermometer: Device = thermometer.into();
        assert_eq!(device_from_thermometer.name(), "Test Thermometer");
        assert_eq!(
            device_from_thermometer.info(),
            "Thermometer: Test Thermometer - Current Temperature: 25.00°C"
        );
    }

    #[test]
    fn remote_device_test() {}
}
