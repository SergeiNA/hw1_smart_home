use super::device::SmartDevice;
use super::types::Celsius;

pub trait TemperatureSensor: SmartDevice {
    fn new(name: String, initial_temperature: Celsius) -> Self;
    fn current_temperature(&self) -> Celsius;
}

#[derive(Debug, Clone)]
pub struct Thermometer {
    name: String,
    temperature: Celsius,
}

impl SmartDevice for Thermometer {
    fn device_name(&self) -> String {
        self.name.clone()
    }
    fn device_info(&self) -> String {
        format!(
            "Thermometer: {} - Current Temperature: {:.2}°C",
            self.name, self.temperature
        )
    }
}

impl TemperatureSensor for Thermometer {
    fn new(name: String, initial_temperature: Celsius) -> Self {
        Thermometer {
            name,
            temperature: initial_temperature,
        }
    }
    fn current_temperature(&self) -> Celsius {
        self.temperature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermometer_create_test() {
        let thermometer = Thermometer::new("Living Room".to_string(), 22.5 as Celsius);
        assert_eq!(thermometer.device_name(), "Living Room");
        assert_eq!(thermometer.current_temperature(), 22.5 as Celsius);
        assert_eq!(
            thermometer.device_info(),
            "Thermometer: Living Room - Current Temperature: 22.50°C"
        );
    }
}
