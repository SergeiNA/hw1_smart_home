#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TemperatureData {
    pub temperature: f32,
    pub timestamp: u32,
}

impl TemperatureData {
    /// Create a new reading with current timestamp
    pub fn new(temperature: f32) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        Self {
            temperature,
            timestamp,
        }
    }

    /// Serialize to 8-byte array
    pub fn to_bytes(self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.temperature.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        buf
    }

    /// Deserialize from 8-byte array
    pub fn from_bytes(buf: &[u8; 8]) -> Self {
        let temperature = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let timestamp = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        Self {
            temperature,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_reading_serialization() {
        let reading = TemperatureData {
            temperature: 22.5,
            timestamp: 1234567890,
        };

        let bytes = reading.to_bytes();
        let decoded = TemperatureData::from_bytes(&bytes);

        assert_eq!(decoded.temperature, 22.5);
        assert_eq!(decoded.timestamp, 1234567890);
    }

    #[test]
    fn test_temperature_reading_new() {
        let reading = TemperatureData::new(25.0);
        assert_eq!(reading.temperature, 25.0);
        assert!(reading.timestamp > 0);
    }
}
