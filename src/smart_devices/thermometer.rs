use super::types::Celsius;
use crate::traits::Information;

pub trait TemperatureSensor: Information {
    fn current_temperature(&self) -> Celsius;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Thermometer {
    name: String,
    temperature: Celsius,
}

impl Thermometer {
    pub fn new(name: String, initial_temperature: Celsius) -> Self {
        Thermometer {
            name,
            temperature: initial_temperature,
        }
    }
}

impl Information for Thermometer {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn info(&self) -> String {
        format!(
            "Thermometer: {} - Current Temperature: {:.2}°C",
            self.name, self.temperature
        )
    }
}

impl TemperatureSensor for Thermometer {
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
        assert_eq!(thermometer.name(), "Living Room");
        assert_eq!(thermometer.current_temperature(), 22.5 as Celsius);
        assert_eq!(
            thermometer.info(),
            "Thermometer: Living Room - Current Temperature: 22.50°C"
        );
    }
}
