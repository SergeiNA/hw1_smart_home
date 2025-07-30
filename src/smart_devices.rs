pub mod outlet;
pub mod thermometer;
pub mod types;

use crate::traits::Information;
pub use outlet::{Outlet, OutletDevice, OutletState};
pub use thermometer::{TemperatureSensor, Thermometer};
pub use types::{Celsius, Fahrenheit, Kelvin, Watt};

#[derive(Debug, Clone, PartialEq)]
pub enum Device {
    OutletType(Outlet),
    ThermometerType(Thermometer),
    Empty,
}

impl From<Outlet> for Device {
    fn from(outlet: Outlet) -> Self {
        Device::OutletType(outlet)
    }
}

impl From<Thermometer> for Device {
    fn from(thermometer: Thermometer) -> Self {
        Device::ThermometerType(thermometer)
    }
}

impl Information for Device {
    fn name(&self) -> String {
        match self {
            Device::OutletType(outlet) => outlet.name(),
            Device::ThermometerType(thermometer) => thermometer.name(),
            Device::Empty => "No Device".to_string(),
        }
    }

    fn info(&self) -> String {
        match self {
            Device::OutletType(outlet) => outlet.info(),
            Device::ThermometerType(thermometer) => thermometer.info(),
            Device::Empty => "No device information available".to_string(),
        }
    }
}

impl Device {
    pub fn new_outlet(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        Device::OutletType(Outlet::new(name, initial_state, power_usage))
    }

    pub fn new_thermometer(name: String, initial_temperature: Celsius) -> Self {
        Device::ThermometerType(Thermometer::new(name, initial_temperature))
    }
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
            let t: &Thermometer = match thermometer {
                Device::ThermometerType(ref t) => t,
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
            let outlet: &mut Outlet = match outlet_device {
                Device::OutletType(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.switch();
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
            let outlet: &mut Outlet = match outlet_device {
                Device::OutletType(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.turn_off();
        }
        {
            let outlet: &Outlet = match outlet_device {
                Device::OutletType(ref o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::Off);
        }
        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: Off, Power Usage: 0 Watt"
        );
        {
            let outlet: &mut Outlet = match outlet_device {
                Device::OutletType(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.turn_on();
        }
        {
            let outlet: &Outlet = match outlet_device {
                Device::OutletType(ref o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
        }
        assert_eq!(
            outlet_device.info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
    }

    #[test]
    fn device_from_test() {
        let outlet = Outlet::new("Test Outlet".to_string(), OutletState::On, 200);
        let device_from_outlet: Device = outlet.into();
        assert_eq!(device_from_outlet.name(), "Test Outlet");
        assert_eq!(
            device_from_outlet.info(),
            "Smart Outlet: Test Outlet - Current State: On, Power Usage: 200 Watt"
        );

        let thermometer = Thermometer::new("Test Thermometer".to_string(), 25.0 as Celsius);
        let device_from_thermometer: Device = thermometer.into();
        assert_eq!(device_from_thermometer.name(), "Test Thermometer");
        assert_eq!(
            device_from_thermometer.info(),
            "Thermometer: Test Thermometer - Current Temperature: 25.00°C"
        );
    }
}
