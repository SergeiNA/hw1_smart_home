# Smart Home Remote Devices - Technical Implementation Guide

## Overview

This document describes the technical implementation approach for **hw3_smart_home_devices** - adding remote interaction capabilities to smart devices through TCP (outlet) and UDP (thermometer) communication, with device simulators running in separate threads.

## Project Goal

Implement remote interaction logic for smart devices and create device simulators that spawn in separate threads to imitate real device behavior over the network.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Main Application                      │
│  ┌────────────────┐              ┌──────────────────┐       │
│  │  SmartHome     │              │   SmartRoom      │       │
│  │  ├─ Room1      │──────────────│   ├─ Outlet1    │       │
│  │  ├─ Room2      │              │   ├─ Outlet2    │       │
│  │  └─ Room3      │              │   └─ Thermometer│       │
│  └────────────────┘              └──────────────────┘       │
└─────────────────────────────────────────────────────────────┘
          │                                │
          │ TCP (sync)                     │ UDP (async)
          ▼                                ▼
┌──────────────────────┐       ┌─────────────────────────┐
│  OutletSimulator     │       │  ThermometerSimulator   │
│  (separate thread)   │       │  (separate thread)      │
│  - TCP Server        │       │  - UDP Sender           │
│  - Non-blocking I/O  │       │  - Random temp values   │
│  - Multi-client      │       │  - Periodic sending     │
└──────────────────────┘       └─────────────────────────┘
```

---

## Implementation Strategy

### Phase 1: Protocol Design

Define communication protocols for each device type:

#### TCP Protocol for Smart Outlet (Binary)

**Protocol Overview:**
- Binary protocol using enum serialization
- Request/Response message types
- Fixed-length header + variable-length payload

**Message Structure:**
```
[0]: u8 message type tag
[1..N]: message payload (varies by type)
```

**Rust Protocol Types:**

```rust
// File: src/protocol/outlet_mock

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutletRequest {
    GetState,
    GetPower,
    TurnOn,
    TurnOff,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutletResponse {
    State(OutletState),           // ON or OFF
    Power(u32),                   // Watts
    Ok,                           // Command succeeded
    Error(String),                // Error message
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OutletState {
    On,
    Off,
}
```

**Wire Format (using bincode):**
```
Request examples (bytes):
  GetState:  [0x00]
  GetPower:  [0x01]
  TurnOn:    [0x02]
  TurnOff:   [0x03]
  Switch:    [0x04]

Response examples (bytes):
  State(On):        [0x00, 0x00]
  State(Off):       [0x00, 0x01]
  Power(150):       [0x01, 0x96, 0x00, 0x00, 0x00]
  Ok:               [0x02]
  Error("msg"):     [0x03, len, ...string_bytes...]
```

**Serialization approach:**
- Use `bincode` crate for efficient binary serialization
- Each message is length-prefixed for framing
- Format: `[4 bytes length][N bytes bincode data]`

#### UDP Protocol for Smart Thermometer (Binary)

**Protocol Overview:**
- Simple fixed-size binary format
- No request/response - one-way push from simulator to device

**Message Structure:**
```
Server → Client (8 bytes):
  [0-3]: f32 temperature in Celsius (little-endian)
  [4-7]: u32 timestamp (seconds since epoch, little-endian)
```

**Rust Types:**

```rust
// File: src/protocol/thermometer_mock

#[repr(C)]
pub struct TemperatureReading {
    pub temperature: f32,  // Celsius
    pub timestamp: u32,    // Unix timestamp
}

impl TemperatureReading {
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.temperature.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; 8]) -> Self {
        let temperature = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let timestamp = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        Self { temperature, timestamp }
    }
}
```

---

### Phase 2: Library Modifications

First, create the protocol module:

**File:** `src/protocol/mod.rs`

```rust
pub mod outlet;
pub mod thermometer;

pub use outlet::{OutletRequest, OutletResponse, OutletState};
pub use thermometer::TemperatureReading;
```

**File:** `src/protocol/outlet.rs`

```rust
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutletRequest {
    GetState,
    GetPower,
    TurnOn,
    TurnOff,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutletResponse {
    State(OutletState),
    Power(u32),
    Ok,
    Error(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutletState {
    On,
    Off,
}

impl OutletRequest {
    /// Serialize request and send over stream
    pub fn send<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let encoded = bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Send length prefix (4 bytes)
        let len = encoded.len() as u32;
        writer.write_all(&len.to_le_bytes())?;

        // Send data
        writer.write_all(&encoded)?;
        writer.flush()?;
        Ok(())
    }

    /// Receive and deserialize request from stream
    pub fn receive<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read data
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        // Deserialize
        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl OutletResponse {
    /// Serialize response and send over stream
    pub fn send<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let encoded = bincode::serialize(self)
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

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
```

**File:** `src/protocol/thermometer.rs`

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TemperatureReading {
    pub temperature: f32,
    pub timestamp: u32,
}

impl TemperatureReading {
    pub fn new(temperature: f32) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        Self { temperature, timestamp }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.temperature.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        buf
    }

    pub fn from_bytes(buf: &[u8; 8]) -> Self {
        let temperature = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let timestamp = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        Self { temperature, timestamp }
    }
}
```

#### 2.1 Smart Outlet - Add TCP Communication

**File:** `src/smart_devices/outlet.rs`

**New structures:**

```rust
use crate::protocol::outlet::{OutletRequest, OutletResponse};
use std::net::TcpStream;
use std::time::Duration;

// Communication mode enum
pub enum OutletMode {
    Local,                          // Direct in-memory access (current behavior)
    Remote { tcp_addr: String },    // TCP communication
}

// Modified Outlet struct
pub struct Outlet {
    name: String,
    mode: OutletMode,
    // For Local mode:
    state: OutletState,
    power_usage: Watt,
    // For Remote mode:
    tcp_connection: Option<TcpStream>,
}
```

**Implementation approach:**

1. Add `mode` field to `Outlet::new()` constructor (or create separate constructors):
   ```rust
   impl Outlet {
       // Existing constructor - local mode
       pub fn new(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
           Outlet {
               name,
               mode: OutletMode::Local,
               state: initial_state,
               power_usage,
               tcp_connection: None,
           }
       }

       // New constructor - remote mode
       pub fn new_remote(name: String, tcp_addr: String) -> Self {
           Outlet {
               name,
               mode: OutletMode::Remote { tcp_addr },
               state: OutletState::Off,  // Default, will be fetched
               power_usage: 0,           // Default, will be fetched
               tcp_connection: None,     // Lazy connection
           }
       }
   }
   ```

2. Modify trait methods to check mode:
   ```rust
   impl OutletDevice for Outlet {
       fn state(&self) -> Result<OutletState, OutletError> {
           match &self.mode {
               OutletMode::Local => Ok(self.state),
               OutletMode::Remote { .. } => {
                   let response = self.send_request(OutletRequest::GetState)?;
                   match response {
                       OutletResponse::State(state) => Ok(state),
                       OutletResponse::Error(e) => Err(OutletError::ProtocolError(e)),
                       _ => Err(OutletError::ProtocolError("Unexpected response".to_string())),
                   }
               }
           }
       }

       fn turn_on(&mut self) -> Result<(), OutletError> {
           match &self.mode {
               OutletMode::Local => {
                   self.state = OutletState::On;
                   Ok(())
               }
               OutletMode::Remote { .. } => {
                   let response = self.send_request(OutletRequest::TurnOn)?;
                   match response {
                       OutletResponse::Ok => Ok(()),
                       OutletResponse::Error(e) => Err(OutletError::ProtocolError(e)),
                       _ => Err(OutletError::ProtocolError("Unexpected response".to_string())),
                   }
               }
           }
       }

       fn power_usage(&self) -> Result<Watt, OutletError> {
           match &self.mode {
               OutletMode::Local => {
                   Ok(if self.state == OutletState::On { self.power_usage } else { 0 })
               }
               OutletMode::Remote { .. } => {
                   let response = self.send_request(OutletRequest::GetPower)?;
                   match response {
                       OutletResponse::Power(watts) => Ok(watts),
                       OutletResponse::Error(e) => Err(OutletError::ProtocolError(e)),
                       _ => Err(OutletError::ProtocolError("Unexpected response".to_string())),
                   }
               }
           }
       }

       // Similar for turn_off() and switch()...
   }
   ```

3. Add helper method for TCP communication:
   ```rust
   impl Outlet {
       fn send_request(&mut self, request: OutletRequest) -> Result<OutletResponse, OutletError> {
           // Get or establish connection
           let stream = self.get_or_connect()?;

           // Send request
           request.send(stream).map_err(OutletError::NetworkError)?;

           // Receive response
           let response = OutletResponse::receive(stream)
               .map_err(OutletError::NetworkError)?;

           Ok(response)
       }

       fn get_or_connect(&mut self) -> Result<&mut TcpStream, OutletError> {
           if self.tcp_connection.is_none() {
               if let OutletMode::Remote { tcp_addr } = &self.mode {
                   let mut stream = TcpStream::connect(tcp_addr)
                       .map_err(OutletError::NetworkError)?;

                   // Set timeouts
                   stream.set_read_timeout(Some(Duration::from_secs(5)))
                       .map_err(OutletError::NetworkError)?;
                   stream.set_write_timeout(Some(Duration::from_secs(5)))
                       .map_err(OutletError::NetworkError)?;

                   self.tcp_connection = Some(stream);
               }
           }

           self.tcp_connection.as_mut()
               .ok_or(OutletError::NetworkError(
                   std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected")
               ))
       }

       fn reconnect(&mut self) -> Result<(), OutletError> {
           self.tcp_connection = None;
           self.get_or_connect()?;
           Ok(())
       }
   }
   ```

4. Connection management:
   - Lazy connection: establish on first command
   - Reconnection strategy: retry on connection failure
   - Timeout handling: 5 seconds for read/write

**Error handling:**
- Return `Result<T, OutletError>` from all methods that communicate over TCP
- Define custom error type:
  ```rust
  #[derive(Debug)]
  pub enum OutletError {
      NetworkError(std::io::Error),
      ProtocolError(String),
      TimeoutError,
  }

  impl std::fmt::Display for OutletError {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          match self {
              OutletError::NetworkError(e) => write!(f, "Network error: {}", e),
              OutletError::ProtocolError(e) => write!(f, "Protocol error: {}", e),
              OutletError::TimeoutError => write!(f, "Timeout error"),
          }
      }
  }

  impl std::error::Error for OutletError {}
  ```

#### 2.2 Smart Thermometer - Add UDP Reception

**File:** `src/smart_devices/thermometer.rs`

**New structures:**

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::net::UdpSocket;

pub enum ThermometerMode {
    Local,                          // Direct value (current behavior)
    Remote { udp_addr: String },    // UDP reception in background thread
}

pub struct Thermometer {
    name: String,
    mode: ThermometerMode,
    // Shared state between main thread and UDP receiver thread:
    temperature: Arc<Mutex<Celsius>>,
    // Thread handle for cleanup:
    receiver_thread: Option<thread::JoinHandle<()>>,
    // Channel to signal thread shutdown:
    shutdown_tx: Option<std::sync::mpsc::Sender<()>>,
}
```

**Implementation approach:**

1. Add constructors:
   - `Thermometer::new_local(name, initial_temp)` - current implementation
   - `Thermometer::new_remote(name, udp_addr)` - UDP-based implementation

2. For remote mode, spawn background thread in constructor:
   ```rust
   impl Thermometer {
       pub fn new_remote(name: String, udp_addr: String) -> Self {
           let temperature = Arc::new(Mutex::new(20.0)); // Default value
           let temp_clone = Arc::clone(&temperature);
           let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

           let receiver_thread = thread::spawn(move || {
               Self::udp_receiver_loop(udp_addr, temp_clone, shutdown_rx);
           });

           Thermometer {
               name,
               mode: ThermometerMode::Remote { udp_addr },
               temperature,
               receiver_thread: Some(receiver_thread),
               shutdown_tx: Some(shutdown_tx),
           }
       }

       fn udp_receiver_loop(
           udp_addr: String,
           temperature: Arc<Mutex<Celsius>>,
           shutdown_rx: std::sync::mpsc::Receiver<()>
       ) {
           let socket = UdpSocket::bind(&udp_addr).expect("Failed to bind UDP socket");
           socket.set_nonblocking(true).expect("Failed to set non-blocking");

           let mut buf = [0u8; 8];

           loop {
               // Check for shutdown signal
               if shutdown_rx.try_recv().is_ok() {
                   break;
               }

               // Try to receive UDP packet
               match socket.recv(&mut buf) {
                   Ok(8) => {
                       // Parse temperature (first 4 bytes as f32)
                       let temp = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                       let mut temp_guard = temperature.lock().unwrap();
                       *temp_guard = temp as Celsius;
                   }
                   Ok(_) => {
                       // Invalid packet size
                   }
                   Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                       // No data available, sleep briefly
                       thread::sleep(std::time::Duration::from_millis(100));
                   }
                   Err(_) => {
                       // Other error
                   }
               }
           }
       }
   }
   ```

3. Implement `current_temperature()`:
   ```rust
   impl TemperatureSensor for Thermometer {
       fn current_temperature(&self) -> Celsius {
           match &self.mode {
               ThermometerMode::Local => {
                   *self.temperature.lock().unwrap()
               }
               ThermometerMode::Remote { .. } => {
                   *self.temperature.lock().unwrap()
               }
           }
       }
   }
   ```

4. Implement `Drop` trait to clean up thread:
   ```rust
   impl Drop for Thermometer {
       fn drop(&mut self) {
           if let Some(tx) = self.shutdown_tx.take() {
               let _ = tx.send(()); // Signal shutdown
           }
           if let Some(handle) = self.receiver_thread.take() {
               let _ = handle.join(); // Wait for thread to finish
           }
       }
   }
   ```

**Important notes:**
- The `Thermometer` can no longer derive `Clone` if it contains thread handles
- Alternative: use `Arc<Thermometer>` for sharing between rooms/devices
- Or: make thread management external to the struct

---

### Phase 3: Simulators Implementation

#### 3.1 Smart Outlet Simulator (TCP Server)

**File:** `src/bin/outlet_simulator.rs`

**Purpose:** Simulate a physical smart outlet by running a TCP server that responds to commands.

**Implementation:**

```rust
use smart_home::protocol::outlet::{OutletRequest, OutletResponse, OutletState};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

struct SimulatedOutlet {
    state: OutletState,
    power_watts: u32,
}

impl SimulatedOutlet {
    fn new(power_watts: u32) -> Self {
        Self {
            state: OutletState::Off,
            power_watts,
        }
    }

    fn handle_request(&mut self, request: OutletRequest) -> OutletResponse {
        match request {
            OutletRequest::GetState => {
                OutletResponse::State(self.state)
            }
            OutletRequest::GetPower => {
                let power = match self.state {
                    OutletState::On => self.power_watts,
                    OutletState::Off => 0,
                };
                OutletResponse::Power(power)
            }
            OutletRequest::TurnOn => {
                self.state = OutletState::On;
                println!("  → Outlet turned ON");
                OutletResponse::Ok
            }
            OutletRequest::TurnOff => {
                self.state = OutletState::Off;
                println!("  → Outlet turned OFF");
                OutletResponse::Ok
            }
            OutletRequest::Switch => {
                self.state = match self.state {
                    OutletState::On => OutletState::Off,
                    OutletState::Off => OutletState::On,
                };
                println!("  → Outlet switched to {:?}", self.state);
                OutletResponse::Ok
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, outlet: Arc<Mutex<SimulatedOutlet>>) {
    println!("  Client connected from: {}", stream.peer_addr().unwrap());

    loop {
        // Receive request
        let request = match OutletRequest::receive(&mut stream) {
            Ok(req) => {
                println!("  Received request: {:?}", req);
                req
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("  Client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("  Error receiving request: {}", e);
                break;
            }
        };

        // Handle request
        let response = {
            let mut outlet = outlet.lock().unwrap();
            outlet.handle_request(request)
        };

        println!("  Sending response: {:?}", response);

        // Send response
        if let Err(e) = response.send(&mut stream) {
            eprintln!("  Error sending response: {}", e);
            break;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <tcp_address> <power_watts>", args[0]);
        eprintln!("Example: {} 127.0.0.1:8001 150", args[0]);
        std::process::exit(1);
    }

    let addr = &args[1];
    let power_watts: u32 = args[2].parse()
        .expect("Invalid power_watts argument");

    let outlet = Arc::new(Mutex::new(SimulatedOutlet::new(power_watts)));

    let listener = TcpListener::bind(addr)
        .expect("Failed to bind TCP listener");

    println!("╔════════════════════════════════════════════╗");
    println!("║   Smart Outlet Simulator (Binary TCP)     ║");
    println!("╚════════════════════════════════════════════╝");
    println!("Listening on: {}", addr);
    println!("Power rating: {} watts", power_watts);
    println!("Waiting for connections...\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let outlet_clone = Arc::clone(&outlet);
                thread::spawn(move || {
                    handle_client(stream, outlet_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}
```

**Features:**
- Binary TCP protocol using type-safe enums
- Thread-per-client model for concurrent connections
- Shared state protected by `Arc<Mutex<>>`
- Proper serialization/deserialization with bincode
- Command-line configurable address and power rating

#### 3.2 Smart Thermometer Simulator (UDP Sender)

**File:** `src/bin/thermometer_simulator.rs`

**Purpose:** Simulate a physical thermometer by sending random temperature values via UDP.

**Configuration file format:**

**File:** `thermometer_config.toml`
```toml
[thermometer]
target_address = "127.0.0.1:9001"
send_interval_ms = 1000  # Send every 1 second
min_temperature = 18.0
max_temperature = 26.0
```

**Implementation:**

```rust
use smart_home::protocol::thermometer::TemperatureReading;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
struct ThermometerConfig {
    target_address: String,
    send_interval_ms: u64,
    min_temperature: f32,
    max_temperature: f32,
}

fn read_config(path: &str) -> ThermometerConfig {
    let content = std::fs::read_to_string(path)
        .expect("Failed to read config file");
    toml::from_str(&content)
        .expect("Failed to parse config file")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "thermometer_config.toml"
    };

    let config = read_config(config_path);

    let socket = UdpSocket::bind("0.0.0.0:0")
        .expect("Failed to create UDP socket");

    println!("╔════════════════════════════════════════════╗");
    println!("║  Smart Thermometer Simulator (Binary UDP) ║");
    println!("╚════════════════════════════════════════════╝");
    println!("Sending to: {}", config.target_address);
    println!("Interval: {}ms", config.send_interval_ms);
    println!("Temperature range: {:.1}°C - {:.1}°C\n",
             config.min_temperature, config.max_temperature);

    let mut rng = rand::thread_rng();
    let mut current_temp = rng.gen_range(config.min_temperature..config.max_temperature);

    loop {
        // Generate realistic temperature (small random walk)
        let change = rng.gen_range(-0.5..0.5);
        current_temp = (current_temp + change)
            .max(config.min_temperature)
            .min(config.max_temperature);

        // Create reading with timestamp
        let reading = TemperatureReading::new(current_temp);
        let buf = reading.to_bytes();

        // Send UDP packet
        match socket.send_to(&buf, &config.target_address) {
            Ok(_) => println!("[{}] Sent: {:.2}°C",
                             reading.timestamp, reading.temperature),
            Err(e) => eprintln!("Failed to send: {}", e),
        }

        thread::sleep(Duration::from_millis(config.send_interval_ms));
    }
}
```

**Features:**
- Reads configuration from TOML file
- Generates realistic temperature values (random walk)
- Non-blocking UDP sending
- Periodic transmission with configurable interval

**Dependencies to add to `Cargo.toml`:**
```toml
[dependencies]
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
bincode = "1.3"  # For binary serialization of enums
```

---

### Phase 4: Example Application

**File:** `examples/remote_devices.rs`

**Purpose:** Demonstrate the complete system with simulators.

**Implementation approach:**

```rust
use smart_home::smart_devices::{Device, OutletState};
use smart_home::smart_room::SmartRoom;
use smart_home::smart_home::SmartHome;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Smart Home Remote Devices Example ===\n");

    // Create devices in remote mode
    let living_room_outlet = Device::new_outlet_remote(
        "Living Room Outlet".to_string(),
        "127.0.0.1:8001".to_string()
    );

    let kitchen_outlet = Device::new_outlet_remote(
        "Kitchen Outlet".to_string(),
        "127.0.0.1:8002".to_string()
    );

    let living_room_thermometer = Device::new_thermometer_remote(
        "Living Room Thermometer".to_string(),
        "127.0.0.1:9001".to_string()
    );

    let bedroom_thermometer = Device::new_thermometer_remote(
        "Bedroom Thermometer".to_string(),
        "127.0.0.1:9002".to_string()
    );

    // Create rooms
    let mut living_room_devices = HashMap::new();
    living_room_devices.insert("outlet".to_string(), living_room_outlet);
    living_room_devices.insert("thermometer".to_string(), living_room_thermometer);
    let living_room = SmartRoom::new("Living Room".to_string(), living_room_devices);

    let mut kitchen_devices = HashMap::new();
    kitchen_devices.insert("outlet".to_string(), kitchen_outlet);
    let kitchen = SmartRoom::new("Kitchen".to_string(), kitchen_devices);

    let mut bedroom_devices = HashMap::new();
    bedroom_devices.insert("thermometer".to_string(), bedroom_thermometer);
    let bedroom = SmartRoom::new("Bedroom".to_string(), bedroom_devices);

    // Create home
    let mut rooms = HashMap::new();
    rooms.insert("Living Room".to_string(), living_room);
    rooms.insert("Kitchen".to_string(), kitchen);
    rooms.insert("Bedroom".to_string(), bedroom);
    let home = SmartHome::new("My Smart Home".to_string(), rooms);

    // Wait for UDP receiver threads to get initial data
    println!("Waiting for device connections...");
    thread::sleep(Duration::from_secs(2));

    // Print initial report
    println!("\n--- Initial Home Report ---");
    match home.generate_report() {
        Ok(report) => println!("{}", report),
        Err(e) => eprintln!("Error generating report: {}", e),
    }

    // Control devices
    println!("\n--- Turning on Living Room Outlet ---");
    // Access device and turn on
    // ...

    thread::sleep(Duration::from_secs(2));

    // Print updated report
    println!("\n--- Updated Home Report ---");
    match home.generate_report() {
        Ok(report) => println!("{}", report),
        Err(e) => eprintln!("Error generating report: {}", e),
    }
}
```

---

### Phase 5: Testing Strategy

#### Unit Tests

1. **Protocol tests:**
   - Test TCP command parsing
   - Test UDP packet serialization/deserialization

2. **Simulator tests:**
   - Test outlet state transitions
   - Test temperature generation

3. **Device tests:**
   - Test local mode (existing tests)
   - Test remote mode with mock servers

#### Integration Tests

**File:** `tests/integration_test.rs`

```rust
use std::thread;
use std::time::Duration;

#[test]
fn test_outlet_simulator_communication() {
    // Spawn outlet simulator in thread
    let simulator_thread = thread::spawn(|| {
        // Run outlet simulator
    });

    thread::sleep(Duration::from_millis(100));

    // Create remote outlet
    let outlet = Outlet::new_remote("Test".to_string(), "127.0.0.1:8999".to_string());

    // Test commands
    outlet.turn_on();
    assert_eq!(outlet.state(), OutletState::On);

    // Cleanup
    drop(outlet);
    // Stop simulator thread
}

#[test]
fn test_thermometer_simulator_communication() {
    // Similar approach for thermometer
}
```

---

## File Structure

```
smart_home/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── traits.rs
│   ├── smart_home.rs
│   ├── smart_room.rs
│   ├── smart_devices.rs
│   ├── smart_devices/
│   │   ├── outlet.rs          # Modified: add TCP binary protocol support
│   │   ├── thermometer.rs     # Modified: add UDP binary protocol support
│   │   └── types.rs
│   ├── protocol/              # New: protocol definitions
│   │   ├── mod.rs
│   │   ├── outlet.rs          # Binary TCP protocol for outlet
│   │   └── thermometer.rs     # Binary UDP protocol for thermometer
│   └── bin/
│       ├── outlet_simulator.rs        # New: TCP server with binary protocol
│       └── thermometer_simulator.rs   # New: UDP sender with binary protocol
├── examples/
│   ├── basic_usage.rs         # Existing
│   └── remote_devices.rs      # New: demonstrate simulators
├── tests/
│   └── integration_test.rs    # New: integration tests
├── thermometer_config.toml    # New: thermometer config
└── .claude/
    └── architecture.md        # This file
```

---

## Running the System

### Step 1: Start Simulators

Terminal 1 - Living Room Outlet (150W):
```bash
cargo run --bin outlet_simulator 127.0.0.1:8001 150
```

Terminal 2 - Kitchen Outlet (300W):
```bash
cargo run --bin outlet_simulator 127.0.0.1:8002 300
```

Terminal 3 - Thermometer (create config first):
```bash
# Create thermometer_config.toml with target "127.0.0.1:9001"
cargo run --bin thermometer_simulator thermometer_config.toml
```

### Step 2: Run Example Application

Terminal 4:
```bash
cargo run --example remote_devices
```

---

## Error Handling Strategy

### Outlet (TCP) Errors

1. **Connection failures:**
   - Retry connection with exponential backoff
   - Maximum retry attempts: 3
   - Timeout: 5 seconds

2. **Protocol errors:**
   - Return descriptive error messages
   - Log unexpected responses

3. **Timeout errors:**
   - Set read/write timeouts to 2 seconds
   - Return timeout error to caller

### Thermometer (UDP) Errors

1. **Binding failures:**
   - Panic in constructor (simulator not running)
   - Alternative: return `Result<Thermometer, Error>`

2. **No data received:**
   - Return last known value
   - Add `last_updated` timestamp field
   - Optional: return error if data is stale (> 10 seconds)

3. **Malformed packets:**
   - Ignore and wait for next packet
   - Log warning

---

## Potential Improvements

1. **Protocol versioning:**
   - Add version byte to UDP packets
   - Add version to TCP handshake

2. **Security:**
   - Add authentication tokens
   - Encrypt communication (TLS for TCP, DTLS for UDP)

3. **Device discovery:**
   - mDNS/Zeroconf for automatic device discovery
   - Broadcast UDP for device announcement

4. **Reliability:**
   - TCP keepalive for outlet connections
   - UDP acknowledgment mechanism for thermometer

5. **Performance:**
   - Connection pooling for outlets
   - Batch commands to reduce round-trips

6. **Monitoring:**
   - Add metrics (command latency, packet loss)
   - Health check endpoints

---

## Summary

This implementation provides:

1. **Backward compatibility:** Existing code continues to work with local mode
2. **Separation of concerns:** Simulators are independent binaries
3. **Thread safety:** Proper synchronization for shared state
4. **Testability:** Both local and remote modes can be tested
5. **Extensibility:** Easy to add new device types or protocols

The key architectural decisions:

- **Mode pattern:** Each device has Local/Remote mode
- **Thread-per-simulator:** Simple concurrency model
- **Text protocol for TCP:** Easy debugging
- **Binary protocol for UDP:** Efficient transmission
- **Arc/Mutex for shared state:** Thread-safe state management
- **Drop trait for cleanup:** Automatic resource cleanup

This architecture allows you to gradually migrate from local to remote devices while maintaining a clean separation between device logic and network communication.
