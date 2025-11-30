use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::io::{Error, Read, Write};

const MAX_MESSAGE_SIZE: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum OutletRequest {
    GetState,
    GetPower,
    TurnOn,
    TurnOff,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum OutletResponse {
    State(OutletState),
    Power(u32),
    Ok,
    Error(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum OutletState {
    On,
    Off,
}

impl OutletRequest {
    /// Serialize request and send over stream
    pub fn send<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let encoded = bincode::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Send length prefix (4 bytes)
        let len = encoded.len() as u32;
        writer.write_all(&len.to_le_bytes())?;

        // Send data
        writer.write_all(&encoded)?;
        writer.flush()?;
        Ok(())
    }

    /// Receive and deserialize request from stream
    pub fn receive<R: Read>(reader: &mut R) -> Result<Self, Error> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Validate length
        if len > MAX_MESSAGE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message too large",
            ));
        }

        // Read data
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        // Deserialize
        let (decoded, _) = bincode::decode_from_slice(&buf, bincode::config::standard())
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(decoded)
    }
}

impl OutletResponse {
    /// Serialize response and send over stream
    pub fn send<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let encoded = bincode::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let len = encoded.len() as u32;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&encoded)?;
        writer.flush()?;
        Ok(())
    }

    /// Receive and deserialize response from stream
    pub fn receive<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message too large",
            ));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        let (decode, _) = bincode::decode_from_slice(&buf, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(decode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ============ OutletRequest Tests ============

    #[test]
    fn test_request_get_state_roundtrip() {
        let request = OutletRequest::GetState;
        let mut buf = Vec::new();

        request.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletRequest::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletRequest::GetState));
    }

    #[test]
    fn test_request_get_power() {
        let request = OutletRequest::GetPower;
        let mut buf = Vec::new();

        request.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletRequest::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletRequest::GetPower));
    }

    #[test]
    fn test_request_turn_on() {
        let request = OutletRequest::TurnOn;
        let mut buf = Vec::new();

        request.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletRequest::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletRequest::TurnOn));
    }

    #[test]
    fn test_request_turn_off() {
        let request = OutletRequest::TurnOff;
        let mut buf = Vec::new();

        request.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletRequest::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletRequest::TurnOff));
    }

    #[test]
    fn test_request_switch() {
        let request = OutletRequest::Switch;
        let mut buf = Vec::new();

        request.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletRequest::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletRequest::Switch));
    }

    #[test]
    fn test_all_requests() {
        let requests = vec![
            OutletRequest::GetState,
            OutletRequest::GetPower,
            OutletRequest::TurnOn,
            OutletRequest::TurnOff,
            OutletRequest::Switch,
        ];

        for request in requests {
            let mut buf = Vec::new();
            request.send(&mut buf).unwrap();
            let mut cursor = Cursor::new(buf);
            let decoded = OutletRequest::receive(&mut cursor).unwrap();

            assert_eq!(format!("{:?}", request), format!("{:?}", decoded));
        }
    }

    // ============ OutletResponse Tests ============

    #[test]
    fn test_response_state_on() {
        let response = OutletResponse::State(OutletState::On);
        let mut buf = Vec::new();

        response.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletResponse::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletResponse::State(OutletState::On)));
    }

    #[test]
    fn test_response_state_off() {
        let response = OutletResponse::State(OutletState::Off);
        let mut buf = Vec::new();

        response.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletResponse::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletResponse::State(OutletState::Off)));
    }

    #[test]
    fn test_response_power() {
        let test_powers = vec![0, 1, 100, 150, 300, 1000, u32::MAX];

        for power in test_powers {
            let response = OutletResponse::Power(power);
            let mut buf = Vec::new();

            response.send(&mut buf).unwrap();
            let mut cursor = Cursor::new(buf);
            let decoded = OutletResponse::receive(&mut cursor).unwrap();

            match decoded {
                OutletResponse::Power(p) => assert_eq!(p, power),
                _ => panic!("Expected Power response"),
            }
        }
    }

    #[test]
    fn test_response_ok() {
        let response = OutletResponse::Ok;
        let mut buf = Vec::new();

        response.send(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = OutletResponse::receive(&mut cursor).unwrap();

        assert!(matches!(decoded, OutletResponse::Ok));
    }

    #[test]
    fn test_response_error() {
        let test_errors = vec![
            "Test error",
            "Connection failed",
            "",
            "Very long error message with lots of details about what went wrong",
        ];

        for error_msg in test_errors {
            let response = OutletResponse::Error(error_msg.to_string());
            let mut buf = Vec::new();

            response.send(&mut buf).unwrap();
            let mut cursor = Cursor::new(buf);
            let decoded = OutletResponse::receive(&mut cursor).unwrap();

            match decoded {
                OutletResponse::Error(msg) => assert_eq!(msg, error_msg),
                _ => panic!("Expected Error response"),
            }
        }
    }

    // ============ Error Handling Tests ============

    #[test]
    fn test_request_message_too_large() {
        // Create a buffer with invalid length (too large)
        let mut buf = Vec::new();
        buf.extend_from_slice(&(MAX_MESSAGE_SIZE as u32 + 1).to_le_bytes());
        buf.extend_from_slice(&[0u8; 100]); // Some data

        let mut cursor = Cursor::new(buf);
        let result = OutletRequest::receive(&mut cursor);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Message too large"));
    }

    #[test]
    fn test_response_message_too_large() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(MAX_MESSAGE_SIZE as u32 + 1).to_le_bytes());
        buf.extend_from_slice(&[0u8; 100]);

        let mut cursor = Cursor::new(buf);
        let result = OutletResponse::receive(&mut cursor);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_request_malformed_data() {
        // Valid length but garbage data
        let len = 10u32;
        let mut buf = len.to_le_bytes().to_vec();
        buf.extend_from_slice(&[0xff; 10]); // Garbage data

        let mut cursor = Cursor::new(buf);
        let result = OutletRequest::receive(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_response_malformed_data() {
        let len = 10u32;
        let mut buf = len.to_le_bytes().to_vec();
        buf.extend_from_slice(&[0xff; 10]);

        let mut cursor = Cursor::new(buf);
        let result = OutletResponse::receive(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_request_unexpected_eof_on_length() {
        // Only 2 bytes instead of 4 for length prefix
        let buf = vec![0x00, 0x01];
        let mut cursor = Cursor::new(buf);

        let result = OutletRequest::receive(&mut cursor);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn test_request_unexpected_eof_on_data() {
        // Length says 100 bytes but only provide 10
        let mut buf = (100u32).to_le_bytes().to_vec();
        buf.extend_from_slice(&[0u8; 10]);

        let mut cursor = Cursor::new(buf);
        let result = OutletRequest::receive(&mut cursor);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn test_empty_buffer() {
        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);

        let result = OutletRequest::receive(&mut cursor);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    // ============ Protocol Format Tests ============

    #[test]
    fn test_length_prefix_format() {
        let request = OutletRequest::GetState;
        let mut buf = Vec::new();

        request.send(&mut buf).unwrap();

        // First 4 bytes should be length in little-endian
        assert!(buf.len() >= 4);
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;

        // Total buffer size should be 4 (length prefix) + len (data)
        assert_eq!(buf.len(), 4 + len);
    }

    #[test]
    fn test_message_size_boundary() {
        // Test message at exactly MAX_MESSAGE_SIZE
        let error_msg = "x".repeat(MAX_MESSAGE_SIZE - 20); // Leave room for enum overhead
        let response = OutletResponse::Error(error_msg.clone());
        let mut buf = Vec::new();

        response.send(&mut buf).unwrap();

        // If it's under the limit, it should work
        if buf.len() - 4 <= MAX_MESSAGE_SIZE {
            let mut cursor = Cursor::new(buf);
            let decoded = OutletResponse::receive(&mut cursor).unwrap();

            match decoded {
                OutletResponse::Error(msg) => assert_eq!(msg, error_msg),
                _ => panic!("Expected Error response"),
            }
        }
    }

    // ============ Consistency Tests ============

    #[test]
    fn test_multiple_messages_in_sequence() {
        let requests = vec![
            OutletRequest::GetState,
            OutletRequest::TurnOn,
            OutletRequest::GetPower,
            OutletRequest::TurnOff,
        ];

        let mut buf = Vec::new();

        // Send all requests
        for request in &requests {
            request.send(&mut buf).unwrap();
        }

        let mut cursor = Cursor::new(buf);

        // Receive and verify all requests
        for expected in &requests {
            let decoded = OutletRequest::receive(&mut cursor).unwrap();
            assert_eq!(format!("{:?}", decoded), format!("{:?}", expected));
        }

        // Should be at end of buffer
        let mut remaining = Vec::new();
        let bytes_read = cursor.read_to_end(&mut remaining).unwrap();
        assert_eq!(bytes_read, 0, "Should have consumed entire buffer");
    }

    #[test]
    fn test_outlet_state_equality() {
        assert_eq!(OutletState::On, OutletState::On);
        assert_eq!(OutletState::Off, OutletState::Off);
        assert_ne!(OutletState::On, OutletState::Off);
    }

    #[test]
    fn test_response_debug_format() {
        let responses = vec![
            OutletResponse::State(OutletState::On),
            OutletResponse::Power(150),
            OutletResponse::Ok,
            OutletResponse::Error("test".to_string()),
        ];

        for response in responses {
            let debug_str = format!("{:?}", response);
            assert!(!debug_str.is_empty());
        }
    }
}
