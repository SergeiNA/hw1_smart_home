pub mod device;
pub mod outlet;
pub mod thermometer;
pub mod types;

pub use device::SmartDevice;
pub use outlet::{Outlet, OutletDevice, OutletState};
use std::fmt;
pub use thermometer::{TemperatureSensor, Thermometer};
pub use types::{Celsius, Fahrenheit, Kelvin, Watt};

#[derive(Debug, Clone)]
pub enum DeviceType {
    OutletType(Outlet),
    ThermometerType(Thermometer),
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::OutletType(outlet) => write!(
                f,
                "Outlet: {}\n{}",
                outlet.device_name(),
                outlet.device_info()
            ),
            DeviceType::ThermometerType(thermometer) => write!(
                f,
                "Thermometer: {}\n{}",
                thermometer.device_name(),
                thermometer.device_info()
            ),
        }
    }
}

impl SmartDevice for DeviceType {
    fn device_name(&self) -> String {
        match self {
            DeviceType::OutletType(outlet) => outlet.device_name(),
            DeviceType::ThermometerType(thermometer) => thermometer.device_name(),
        }
    }

    fn device_info(&self) -> String {
        match self {
            DeviceType::OutletType(outlet) => outlet.device_info(),
            DeviceType::ThermometerType(thermometer) => thermometer.device_info(),
        }
    }
}

impl DeviceType {
    pub fn new_outlet(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        DeviceType::OutletType(Outlet::new(name, initial_state, power_usage))
    }

    pub fn new_thermometer(name: String, initial_temperature: Celsius) -> Self {
        DeviceType::ThermometerType(Thermometer::new(name, initial_temperature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_outlet_create_test() {
        let outlet = DeviceType::new_outlet("Living Room".to_string(), OutletState::On, 150);
        assert_eq!(outlet.device_name(), "Living Room");
        assert_eq!(
            outlet.device_info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
    }

    #[test]
    fn device_type_thermometer_create_test() {
        let thermometer = DeviceType::new_thermometer("Bedroom".to_string(), 22.5 as Celsius);
        assert_eq!(thermometer.device_name(), "Bedroom");
        assert_eq!(
            thermometer.device_info(),
            "Thermometer: Bedroom - Current Temperature: 22.50°C"
        );
    }

    #[test]
    fn device_type_thermometer_get_test() {
        let thermometer = DeviceType::new_thermometer("Bedroom".to_string(), 22.5 as Celsius);
        assert_eq!(thermometer.device_name(), "Bedroom");
        assert_eq!(
            thermometer.device_info(),
            "Thermometer: Bedroom - Current Temperature: 22.50°C"
        );
        {
            let t: &Thermometer = match thermometer {
                DeviceType::ThermometerType(ref t) => t,
                _ => panic!("Expected ThermometerType"),
            };
            assert_eq!(t.current_temperature(), 22.5 as Celsius);
        }
    }

    #[test]
    fn device_type_outlet_switch_test() {
        let mut outlet_device =
            DeviceType::new_outlet("Living Room".to_string(), OutletState::On, 150);
        assert_eq!(outlet_device.device_name(), "Living Room");
        assert_eq!(
            outlet_device.device_info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
        {
            let outlet: &mut Outlet = match outlet_device {
                DeviceType::OutletType(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.switch();
        }
        assert_eq!(
            outlet_device.device_info(),
            "Smart Outlet: Living Room - Current State: Off, Power Usage: 0 Watt"
        );
    }

    #[test]
    fn device_type_outlet_turn_on_off_test() {
        let mut outlet_device =
            DeviceType::new_outlet("Living Room".to_string(), OutletState::On, 150);
        assert_eq!(outlet_device.device_name(), "Living Room");
        assert_eq!(
            outlet_device.device_info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
        {
            let outlet: &mut Outlet = match outlet_device {
                DeviceType::OutletType(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.turn_off();
        }
        {
            let outlet: &Outlet = match outlet_device {
                DeviceType::OutletType(ref o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::Off);
        }
        assert_eq!(
            outlet_device.device_info(),
            "Smart Outlet: Living Room - Current State: Off, Power Usage: 0 Watt"
        );
        {
            let outlet: &mut Outlet = match outlet_device {
                DeviceType::OutletType(ref mut o) => o,
                _ => panic!("Expected OutletType"),
            };
            outlet.turn_on();
        }
        {
            let outlet: &Outlet = match outlet_device {
                DeviceType::OutletType(ref o) => o,
                _ => panic!("Expected OutletType"),
            };
            assert_eq!(outlet.state(), OutletState::On);
        }
        assert_eq!(
            outlet_device.device_info(),
            "Smart Outlet: Living Room - Current State: On, Power Usage: 150 Watt"
        );
    }
}
