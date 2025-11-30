use super::types::Watt;
use crate::protocols::outlet::{OutletRequest, OutletResponse};
use crate::smart_devices::OutletDevice;
use crate::smart_devices::errors::OutletRemoteError;
use crate::smart_devices::types::OutletState;
use crate::traits::Information;
use std::cell::RefCell;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
pub struct OutletRemote {
    name: String,
    tcp_connection: RefCell<TcpStream>,
}

impl OutletRemote {
    pub fn new(name: String, tcp_addr: String) -> Result<Self, OutletRemoteError> {
        let stream = TcpStream::connect(tcp_addr).map_err(OutletRemoteError::NetworkError)?;

        // Set timeouts to prevent hanging
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(OutletRemoteError::NetworkError)?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(OutletRemoteError::NetworkError)?;

        Ok(OutletRemote {
            name,
            tcp_connection: RefCell::from(stream),
        })
    }

    fn send_request(&self, request: OutletRequest) -> Result<OutletResponse, OutletRemoteError> {
        let mut stream = self.tcp_connection.borrow_mut();
        let stream: &mut TcpStream = &mut stream;

        // Serialize and send the request
        request
            .send(stream)
            .map_err(OutletRemoteError::NetworkError)?;
        // Receive and deserialize the response
        let response = OutletResponse::receive(stream).map_err(OutletRemoteError::NetworkError)?;
        Ok(response)
    }
}

impl OutletDevice for OutletRemote {
    fn turn_on(&mut self) -> Result<(), OutletRemoteError> {
        let request = OutletRequest::TurnOn;
        let response = self.send_request(request)?;
        match response {
            OutletResponse::Ok => Ok(()),
            OutletResponse::Error(msg) => Err(OutletRemoteError::ProtocolError(msg)),
            _ => Err(OutletRemoteError::ProtocolError(
                "Unexpected response".to_string(),
            )),
        }
    }

    fn turn_off(&mut self) -> Result<(), OutletRemoteError> {
        let request = OutletRequest::TurnOff;
        let response = self.send_request(request)?;
        match response {
            OutletResponse::Ok => Ok(()),
            OutletResponse::Error(msg) => Err(OutletRemoteError::ProtocolError(msg)),
            _ => Err(OutletRemoteError::ProtocolError(
                "Unexpected response".to_string(),
            )),
        }
    }

    fn switch(&mut self) -> Result<(), OutletRemoteError> {
        let request = OutletRequest::Switch;
        let response = self.send_request(request)?;
        match response {
            OutletResponse::Ok => Ok(()),
            OutletResponse::Error(msg) => Err(OutletRemoteError::ProtocolError(msg)),
            _ => Err(OutletRemoteError::ProtocolError(
                "Unexpected response".to_string(),
            )),
        }
    }
    fn state(&self) -> Result<OutletState, OutletRemoteError> {
        let request = OutletRequest::GetState;
        let response = self.send_request(request)?;
        match response {
            OutletResponse::State(state) => Ok(state.into()),
            OutletResponse::Error(msg) => Err(OutletRemoteError::ProtocolError(msg)),
            _ => Err(OutletRemoteError::ProtocolError(
                "Unexpected response".to_string(),
            )),
        }
    }

    fn power_usage(&self) -> Result<Watt, OutletRemoteError> {
        let request = OutletRequest::GetPower;
        let response = self.send_request(request)?;
        match response {
            OutletResponse::Power(power) => Ok(power.into()),
            OutletResponse::Error(msg) => Err(OutletRemoteError::ProtocolError(msg)),
            _ => Err(OutletRemoteError::ProtocolError(
                "Unexpected response".to_string(),
            )),
        }
    }
}

impl Information for OutletRemote {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn info(&self) -> String {
        let usage = match OutletDevice::power_usage(self) {
            Ok(power) => power,
            Err(e) => {
                return format!(
                    "Remote Smart Outlet {} - Current State: Unknown, Power Usage: Unknown Watt. Error: {}",
                    self.name, e,
                );
            }
        };

        let state = match OutletDevice::state(self) {
            Ok(state) => state,
            Err(e) => {
                return format!(
                    "Remote Smart Outlet {} - Current State: Unknown, Power Usage: {} Watt. Error: {}",
                    self.name, usage, e,
                );
            }
        };

        format!(
            "Remote Smart Outlet: {} - Current State: {}, Power Usage: {} Watt",
            self.name, state, usage
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::outlet::OutletState as RemoteOutletState;
    use std::net::TcpListener;
    use std::thread;

    /// Helper function to create a mock TCP server that responds to outlet protocol
    fn start_mock_outlet_server(addr: &str) -> TcpListener {
        let listener = TcpListener::bind(addr).expect("Failed to bind test server");
        listener
    }

    /// Mock server that handles a single request-response cycle
    fn mock_server_handler<F>(listener: TcpListener, handler: F)
    where
        F: FnOnce(&mut TcpStream) + Send + 'static,
    {
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                handler(&mut stream);
            }
        });
    }

    #[test]
    fn test_outlet_remote_new_connection() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |_stream| {
            // Just accept connection
        });

        let outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string());

        assert!(outlet.is_ok());
        let outlet = outlet.unwrap();
        assert_eq!(outlet.name(), "Test Outlet");
    }

    #[test]
    fn test_outlet_remote_connection_refused() {
        // Try to connect to a port that's not listening
        let result = OutletRemote::new("Test Outlet".to_string(), "127.0.0.1:65534".to_string());

        assert!(result.is_err());
        match result {
            Err(OutletRemoteError::NetworkError(_)) => {}
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_outlet_remote_turn_on() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            // Read request
            let request = OutletRequest::receive(stream).unwrap();
            assert!(matches!(request, OutletRequest::TurnOn));

            // Send OK response
            OutletResponse::Ok.send(stream).unwrap();
        });

        let mut outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.turn_on();
        assert!(result.is_ok());
    }

    #[test]
    fn test_outlet_remote_turn_off() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let request = OutletRequest::receive(stream).unwrap();
            assert!(matches!(request, OutletRequest::TurnOff));
            OutletResponse::Ok.send(stream).unwrap();
        });

        let mut outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.turn_off();
        assert!(result.is_ok());
    }

    #[test]
    fn test_outlet_remote_switch() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let request = OutletRequest::receive(stream).unwrap();
            assert!(matches!(request, OutletRequest::Switch));
            OutletResponse::Ok.send(stream).unwrap();
        });

        let mut outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.switch();
        assert!(result.is_ok());
    }

    #[test]
    fn test_outlet_remote_get_state() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let request = OutletRequest::receive(stream).unwrap();
            assert!(matches!(request, OutletRequest::GetState));
            OutletResponse::State(RemoteOutletState::On)
                .send(stream)
                .unwrap();
        });

        let outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.state();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OutletState::On);
    }

    #[test]
    fn test_outlet_remote_get_power() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let request = OutletRequest::receive(stream).unwrap();
            assert!(matches!(request, OutletRequest::GetPower));
            OutletResponse::Power(150).send(stream).unwrap();
        });

        let outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.power_usage();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 150);
    }

    #[test]
    fn test_outlet_remote_protocol_error() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let _request = OutletRequest::receive(stream).unwrap();
            OutletResponse::Error("Device malfunction".to_string())
                .send(stream)
                .unwrap();
        });

        let mut outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.turn_on();
        assert!(result.is_err());
        match result {
            Err(OutletRemoteError::ProtocolError(msg)) => {
                assert_eq!(msg, "Device malfunction");
            }
            _ => panic!("Expected ProtocolError"),
        }
    }

    #[test]
    fn test_outlet_remote_unexpected_response() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let _request = OutletRequest::receive(stream).unwrap();
            // Send State response when Ok was expected
            OutletResponse::State(RemoteOutletState::On)
                .send(stream)
                .unwrap();
        });

        let mut outlet = OutletRemote::new("Test Outlet".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let result = outlet.turn_on();
        assert!(result.is_err());
        match result {
            Err(OutletRemoteError::ProtocolError(msg)) => {
                assert_eq!(msg, "Unexpected response");
            }
            _ => panic!("Expected ProtocolError with 'Unexpected response'"),
        }
    }

    #[test]
    fn test_outlet_remote_info_success() {
        let listener = start_mock_outlet_server("127.0.0.1:0");
        let addr = listener.local_addr().unwrap();

        mock_server_handler(listener, |stream| {
            let mut res_power = false;
            let mut resp_state = false;
            loop {
                if res_power && resp_state {
                    break;
                }
                let request = OutletRequest::receive(stream).unwrap();
                match request {
                    OutletRequest::GetPower => {
                        OutletResponse::Power(100).send(stream).unwrap();
                        res_power = true;
                    }
                    OutletRequest::GetState => {
                        OutletResponse::State(RemoteOutletState::On)
                            .send(stream)
                            .unwrap();
                        resp_state = true;
                    }
                    _ => {}
                }
            }
        });

        let outlet = OutletRemote::new("Living Room".to_string(), addr.to_string()).unwrap();

        thread::sleep(Duration::from_millis(10));
        let info = outlet.info();
        assert_eq!(
            info,
            "Remote Smart Outlet: Living Room - Current State: On, Power Usage: 100 Watt"
        );
    }

    #[test]
    fn test_outlet_remote_state_conversion() {
        assert_eq!(OutletState::from(RemoteOutletState::On), OutletState::On);
        assert_eq!(OutletState::from(RemoteOutletState::Off), OutletState::Off);
    }
}
