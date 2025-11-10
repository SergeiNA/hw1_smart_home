# Step 7: Testing Strategy

**Prerequisites:** All previous steps completed

**Goal:** Implement comprehensive testing for the remote device system

---

## Overview

Testing strategy includes:
- Unit tests for protocol serialization/deserialization
- Integration tests for TCP and UDP communication
- Mock simulators for automated testing
- Property-based tests for protocol robustness

---

## 7.1 Protocol Unit Tests

**File:** `src/protocol/outlet.rs`

Add comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_request_serialization() {
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

    #[test]
    fn test_response_serialization() {
        let responses = vec![
            OutletResponse::State(OutletState::On),
            OutletResponse::State(OutletState::Off),
            OutletResponse::Power(150),
            OutletResponse::Ok,
            OutletResponse::Error("Test error".to_string()),
        ];

        for response in responses {
            let mut buf = Vec::new();
            response.send(&mut buf).unwrap();

            let mut cursor = Cursor::new(buf);
            let decoded = OutletResponse::receive(&mut cursor).unwrap();

            assert_eq!(format!("{:?}", response), format!("{:?}", decoded));
        }
    }

    #[test]
    fn test_message_too_large() {
        // Try to receive a message with invalid length
        let mut buf = vec![0xff, 0xff, 0xff, 0xff]; // Very large length
        let mut cursor = Cursor::new(buf);

        let result = OutletRequest::receive(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_data() {
        // Length is valid but data is garbage
        let len = 10u32;
        let mut buf = len.to_le_bytes().to_vec();
        buf.extend_from_slice(&[0xff; 10]); // Garbage data

        let mut cursor = Cursor::new(buf);
        let result = OutletRequest::receive(&mut cursor);
        assert!(result.is_err());
    }
}
```

---

## 7.2 Integration Tests

**File:** `tests/integration_test.rs`

```rust
use smart_home::protocol::outlet::{OutletRequest, OutletResponse, OutletState};
use smart_home::protocol::thermometer::TemperatureReading;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_outlet_tcp_communication() {
    // Start a simple TCP server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    // Server thread
    let server_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        // Receive request
        let request = OutletRequest::receive(&mut stream).unwrap();
        assert!(matches!(request, OutletRequest::TurnOn));

        // Send response
        let response = OutletResponse::Ok;
        response.send(&mut stream).unwrap();
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    // Client
    let mut stream = TcpStream::connect(addr).unwrap();

    // Send request
    let request = OutletRequest::TurnOn;
    request.send(&mut stream).unwrap();

    // Receive response
    let response = OutletResponse::receive(&mut stream).unwrap();
    assert!(matches!(response, OutletResponse::Ok));

    server_handle.join().unwrap();
}

#[test]
fn test_thermometer_udp_communication() {
    // Create receiver socket
    let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    let receiver_addr = receiver.local_addr().unwrap();

    receiver.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    // Create sender socket
    let sender = UdpSocket::bind("0.0.0.0:0").unwrap();

    // Send temperature reading
    let reading = TemperatureReading::new(22.5);
    let buf = reading.to_bytes();
    sender.send_to(&buf, receiver_addr).unwrap();

    // Receive temperature reading
    let mut recv_buf = [0u8; 8];
    let (size, _) = receiver.recv_from(&mut recv_buf).unwrap();
    assert_eq!(size, 8);

    let received = TemperatureReading::from_bytes(&recv_buf);
    assert_eq!(received.temperature, 22.5);
    assert_eq!(received.timestamp, reading.timestamp);
}

#[test]
fn test_outlet_simulator_full_lifecycle() {
    use smart_home::smart_devices::Outlet;
    use smart_home::smart_devices::outlet::{OutletDevice, OutletState};

    // Start simulator in thread
    let simulator_handle = thread::spawn(|| {
        // This would be the full simulator code
        // For testing, we'll use a simplified version
        run_test_outlet_simulator("127.0.0.1:18001", 150);
    });

    thread::sleep(Duration::from_millis(500));

    // Create remote outlet
    let mut outlet = Outlet::new_remote(
        "Test Outlet".to_string(),
        "127.0.0.1:18001".to_string()
    );

    // Test operations
    assert_eq!(outlet.state().unwrap(), OutletState::Off);
    outlet.turn_on().unwrap();
    assert_eq!(outlet.state().unwrap(), OutletState::On);
    assert_eq!(outlet.power_usage().unwrap(), 150);
    outlet.turn_off().unwrap();
    assert_eq!(outlet.state().unwrap(), OutletState::Off);
    assert_eq!(outlet.power_usage().unwrap(), 0);

    // Cleanup is automatic when outlet is dropped
}

// Helper function for test
fn run_test_outlet_simulator(addr: &str, power_watts: u32) {
    use std::net::TcpListener;

    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(false).unwrap();

    let mut state = OutletState::Off;

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            loop {
                match OutletRequest::receive(&mut stream) {
                    Ok(req) => {
                        let response = match req {
                            OutletRequest::GetState => OutletResponse::State(state),
                            OutletRequest::GetPower => {
                                let power = if state == OutletState::On { power_watts } else { 0 };
                                OutletResponse::Power(power)
                            }
                            OutletRequest::TurnOn => {
                                state = OutletState::On;
                                OutletResponse::Ok
                            }
                            OutletRequest::TurnOff => {
                                state = OutletState::Off;
                                OutletResponse::Ok
                            }
                            OutletRequest::Switch => {
                                state = match state {
                                    OutletState::On => OutletState::Off,
                                    OutletState::Off => OutletState::On,
                                };
                                OutletResponse::Ok
                            }
                        };
                        let _ = response.send(&mut stream);
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
```

---

## 7.3 Mock Simulator for Tests

Create reusable mock simulators:

**File:** `tests/common/mod.rs`

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::{TcpListener, UdpSocket};
use smart_home::protocol::outlet::{OutletRequest, OutletResponse, OutletState};
use smart_home::protocol::thermometer::TemperatureReading;

pub struct MockOutletSimulator {
    addr: String,
    handle: Option<thread::JoinHandle<()>>,
    state: Arc<Mutex<OutletState>>,
}

impl MockOutletSimulator {
    pub fn start(addr: &str, power_watts: u32) -> Self {
        let addr_clone = addr.to_string();
        let state = Arc::new(Mutex::new(OutletState::Off));
        let state_clone = Arc::clone(&state);

        let handle = thread::spawn(move || {
            Self::run_server(&addr_clone, power_watts, state_clone);
        });

        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));

        Self {
            addr: addr.to_string(),
            handle: Some(handle),
            state,
        }
    }

    fn run_server(addr: &str, power_watts: u32, state: Arc<Mutex<OutletState>>) {
        let listener = TcpListener::bind(addr).unwrap();

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                loop {
                    match OutletRequest::receive(&mut stream) {
                        Ok(req) => {
                            let mut current_state = state.lock().unwrap();
                            let response = match req {
                                OutletRequest::GetState => OutletResponse::State(*current_state),
                                OutletRequest::GetPower => {
                                    let power = if *current_state == OutletState::On {
                                        power_watts
                                    } else {
                                        0
                                    };
                                    OutletResponse::Power(power)
                                }
                                OutletRequest::TurnOn => {
                                    *current_state = OutletState::On;
                                    OutletResponse::Ok
                                }
                                OutletRequest::TurnOff => {
                                    *current_state = OutletState::Off;
                                    OutletResponse::Ok
                                }
                                OutletRequest::Switch => {
                                    *current_state = match *current_state {
                                        OutletState::On => OutletState::Off,
                                        OutletState::Off => OutletState::On,
                                    };
                                    OutletResponse::Ok
                                }
                            };
                            let _ = response.send(&mut stream);
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    }

    pub fn current_state(&self) -> OutletState {
        *self.state.lock().unwrap()
    }
}

pub struct MockThermometerSimulator {
    handle: Option<thread::JoinHandle<()>>,
}

impl MockThermometerSimulator {
    pub fn start(target_addr: &str, temperature: f32) -> Self {
        let target_addr = target_addr.to_string();

        let handle = thread::spawn(move || {
            Self::run_sender(&target_addr, temperature);
        });

        Self {
            handle: Some(handle),
        }
    }

    fn run_sender(target_addr: &str, temperature: f32) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        for _ in 0..10 {
            let reading = TemperatureReading::new(temperature);
            let buf = reading.to_bytes();
            let _ = socket.send_to(&buf, target_addr);
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
```

---

## 7.4 Performance Tests

**File:** `tests/performance_test.rs`

```rust
use std::time::Instant;
use smart_home::protocol::outlet::{OutletRequest, OutletResponse};

#[test]
#[ignore] // Run with: cargo test --test performance_test -- --ignored
fn test_serialization_performance() {
    let iterations = 100_000;
    let request = OutletRequest::TurnOn;

    let start = Instant::now();
    for _ in 0..iterations {
        let mut buf = Vec::new();
        request.send(&mut buf).unwrap();
    }
    let duration = start.elapsed();

    println!("Serialized {} requests in {:?}", iterations, duration);
    println!("Average: {:?} per request", duration / iterations);
}

#[test]
#[ignore]
fn test_concurrent_connections() {
    // Test multiple concurrent TCP connections
    // Implementation depends on your needs
}
```

---

## 7.5 Running Tests

### Unit tests:
```bash
cargo test
```

### Integration tests:
```bash
cargo test --test integration_test
```

### All tests including ignored:
```bash
cargo test -- --ignored
cargo test --include-ignored
```

### With output:
```bash
cargo test -- --nocapture
```

### Specific test:
```bash
cargo test test_outlet_tcp_communication
```

---

## 7.6 Test Coverage

Add coverage reporting:

**Install tarpaulin:**
```bash
cargo install cargo-tarpaulin
```

**Run coverage:**
```bash
cargo tarpaulin --out Html --output-dir coverage
```

**View report:**
```bash
open coverage/index.html
```

---

## 7.7 Continuous Integration

**File:** `.github/workflows/test.yml`

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run tests
        run: cargo test --all-features

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check
```

---

## Summary

✅ Created unit tests for protocol serialization
✅ Implemented integration tests for TCP/UDP
✅ Built mock simulators for automated testing
✅ Added performance benchmarks
✅ Set up test coverage reporting
✅ Configured CI/CD pipeline

**Implementation Complete!** All steps finished. Return to [Architecture Overview](architecture.md) for reference.
