# Smart Home System

A Rust-based smart home library with support for local and remote device management, featuring TCP/UDP communication protocols and built-in device simulators.

## 📖 Overview

This project implements a flexible smart home system that manages rooms and devices (outlets and thermometers). It supports both local mock devices and remote devices that communicate over network protocols, making it ideal for testing, development, and real-world deployment scenarios.

### Key Features

- 🏠 **Hierarchical Organization**: Home → Rooms → Devices
- 🔌 **Smart Outlets**: TCP-based outlets with on/off control and power monitoring
- 🌡️ **Smart Thermometers**: UDP-based thermometers with real-time temperature updates
- 🧪 **Built-in Simulators**: Library-based device simulators for easy testing
- 🔄 **Dual Mode Support**: Local mock devices and remote network devices
- 🧵 **Thread-Safe**: Concurrent access to shared device state
- ✅ **Comprehensive Tests**: 119+ tests covering all major functionality

---

## 🏗️ Architecture

### High-Level Design

```
┌────────────────────────────────────────────────────────────────┐
│                      Smart Home Application                     │
│                                                                 │
│  ┌──────────────────┐                                          │
│  │   SmartHome      │                                          │
│  │  ├─ Living Room  │─────┐                                    │
│  │  ├─ Bedroom      │     │                                    │
│  │  └─ Kitchen      │     │                                    │
│  └──────────────────┘     │                                    │
│           │                │                                    │
│           ▼                ▼                                    │
│  ┌──────────────────┬──────────────────┐                      │
│  │   SmartRoom      │   SmartRoom      │                      │
│  │  ├─ Outlet 1     │  ├─ Thermometer │                      │
│  │  └─ Thermometer  │  └─ Outlet 2    │                      │
│  └──────────────────┴──────────────────┘                      │
└────────────────────────────────────────────────────────────────┘
                      │                  │
        ┌─────────────┴──────┬───────────┴──────────┐
        │                    │                       │
        ▼                    ▼                       ▼
┌────────────────┐  ┌────────────────┐    ┌──────────────────┐
│ OutletRemote   │  │ OutletMock     │    │ThermometerRemote │
│ (TCP Client)   │  │ (In-memory)    │    │ (UDP Receiver)   │
└────────────────┘  └────────────────┘    └──────────────────┘
        │                                           │
        │ TCP                                  UDP │
        ▼                                           ▼
┌────────────────┐                        ┌──────────────────┐
│OutletSimulator │                        │ThermometerSim... │
│ (TCP Server)   │                        │ (UDP Sender)     │
│ - Multi-client │                        │ - Temperature    │
│ - Non-blocking │                        │   patterns       │
└────────────────┘                        └──────────────────┘
```

### Core Components

#### 1. Smart Home Hierarchy

- **`SmartHome`**: Top-level container managing multiple rooms
  - Provides unified access to all devices
  - Generates comprehensive status reports
  - Supports dynamic room and device management

- **`SmartRoom`**: Room-level container for devices
  - Groups related devices by location
  - Provides room-specific device access
  - Supports device enumeration and reporting

- **`Device`**: Enum wrapper for all device types
  - `OutletTypeMock` - Local mock outlet
  - `OutletTypeRemote` - TCP-connected outlet
  - `ThermometerTypeMock` - Local mock thermometer
  - `ThermometerTypeRemote` - UDP-connected thermometer

#### 2. Device Types

**Smart Outlet** (implements `OutletDevice` trait)
- Operations: `turn_on()`, `turn_off()`, `switch()`, `state()`, `power_usage()`
- **Local Mode** (`OutletMock`): Direct in-memory access
- **Remote Mode** (`OutletRemote`): Binary TCP protocol
  - Persistent TCP connections
  - Automatic reconnection handling
  - Request/Response message pattern

**Smart Thermometer** (implements `TemperatureSensor` trait)
- Operation: `current_temperature()`
- **Local Mode** (`ThermometerMock`): Returns fixed value
- **Remote Mode** (`ThermometerRemote`): UDP receiver thread
  - Background thread receives temperature updates
  - Always returns latest received value
  - Automatic cleanup via `Drop` trait

#### 3. Communication Protocols

**TCP Protocol (Outlets)**
```
Binary protocol using bincode serialization:
  - Request types: GetState, GetPower, TurnOn, TurnOff, Switch
  - Response types: State(On/Off), Power(u32), Ok, Error(String)
  - Frame format: [4-byte length prefix][bincode payload]
```

**UDP Protocol (Thermometers)**
```
Fixed 8-byte binary format:
  [0-3]: f32 temperature (little-endian)
  [4-7]: u32 timestamp (little-endian)
```

#### 4. Device Simulators

**`OutletSimulator`** - TCP Server
- Spawns in background thread
- Handles multiple concurrent clients
- Maintains outlet state across connections
- Configurable power consumption

**`ThermometerSimulator`** - UDP Sender
- Spawns in background thread
- Supports multiple temperature patterns:
  - `Constant(temp)` - Fixed temperature
  - `RandomWalk { min, max, step }` - Realistic variations
  - `SineWave { center, amplitude, period }` - Periodic oscillations
- Configurable update interval

**`DeviceSimulatorSpawner`** - Unified Management
- Centralized spawning of multiple simulators
- Automatic address management (supports port 0)
- Clean separation of device lifecycles
- Builder pattern configuration

---

## 📁 Project Structure

```
hw1_smart_home/
├── src/
│   ├── lib.rs                      # Main library entry point
│   ├── traits.rs                   # Common traits (Information, etc.)
│   ├── smart_home.rs               # SmartHome implementation
│   ├── smart_room.rs               # SmartRoom implementation
│   │
│   ├── smart_devices/              # Device implementations
│   │   ├── mod.rs
│   │   ├── types.rs                # Common device types
│   │   ├── errors.rs               # Device error types
│   │   ├── outlet_mock.rs          # Local outlet
│   │   ├── outlet_remote.rs        # TCP outlet client
│   │   ├── thermometer_mock.rs     # Local thermometer
│   │   └── thermometer_remote.rs   # UDP thermometer client
│   │
│   ├── protocols/                  # Network protocols
│   │   ├── mod.rs
│   │   ├── outlet.rs               # TCP outlet protocol
│   │   └── thermometer.rs          # UDP thermometer protocol
│   │
│   └── simulators/                 # Device simulators
│       ├── mod.rs
│       ├── spawner.rs              # Unified spawner
│       ├── outlet.rs               # Outlet simulator (TCP server)
│       └── thermometer.rs          # Thermometer simulator (UDP sender)
│
├── examples/
│   ├── basic_usage.rs              # Local mock devices example
│   └── remote_devices.rs           # Remote devices with simulators
│
├── tests/                          # Integration tests
│
└── .claude/                        # Documentation
    ├── architecture.md             # Detailed technical guide
    ├── SIMULATOR_ARCHITECTURE.md   # Simulator design doc
    └── review.md                   # Code review
```

---

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Dependencies

```toml
[dependencies]
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

### Building

```bash
# Build the library and all binaries
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

---

## 💡 Usage Examples

### Example 1: Local Mock Devices

```rust
use smart_home::{create_home, create_room};
use smart_home::smart_devices::{Device, OutletDevice};
use smart_home::smart_devices::outlet_mock::OutletMock;
use smart_home::smart_devices::thermometer_mock::ThermometerMock;
use smart_home::smart_devices::types::OutletState;
use smart_home::traits::Information;

fn main() {
    // Create mock devices
    let outlet = OutletMock::new("Living Room Lamp", OutletState::Off, 60);
    let thermometer = ThermometerMock::new("Living Room Sensor", 22.5);

    // Create home using macros
    let mut home = create_home!(
        "My Smart Home",
        {
            "Living Room",
            create_room!(
                "Living Room",
                "lamp" => Device::from(outlet),
                "sensor" => Device::from(thermometer)
            )
        }
    );

    // Display status
    println!("{}", home.info());
}
```

### Example 2: Remote Devices with Simulators

```rust
use smart_home::{create_home, create_room};
use smart_home::simulators::spawner::DeviceSimulatorSpawner;
use smart_home::simulators::outlet::OutletSimulatorConfig;
use smart_home::simulators::thermometer::{
    ThermometerSimulatorConfig, TemperaturePattern
};
use smart_home::smart_devices::{Device, Watt};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::thermometer_remote::ThermometerRemote;
use std::time::Duration;
use std::thread;

fn main() {
    // Spawn simulators
    let mut spawner = DeviceSimulatorSpawner::new();

    // Spawn outlet simulator on random port
    let outlet_config = OutletSimulatorConfig::new("127.0.0.1:0", 150 as Watt);
    let outlet_addr = spawner
        .spawn_outlet_simulator("TV Outlet".to_string(), outlet_config)
        .expect("Failed to spawn outlet");

    // Spawn thermometer simulator with sine wave pattern
    let thermo_config = ThermometerSimulatorConfig::new(
        "Kitchen Thermometer",
        "127.0.0.1:9030".to_string(),
        Duration::from_millis(100),
    )
    .with_pattern(TemperaturePattern::SineWave {
        center: 22.0,
        amplitude: 3.0,
        period_secs: 10.0,
    });

    let thermo_addr = spawner
        .spawn_thermometer_simulator("Kitchen Thermometer".to_string(), thermo_config)
        .expect("Failed to spawn thermometer");

    // Create remote devices
    let outlet = OutletRemote::new("TV Outlet".to_string(), outlet_addr)
        .expect("Failed to connect outlet");

    let thermometer = ThermometerRemote::new(
        "Kitchen Thermometer".to_string(),
        thermo_addr,
    )
    .expect("Failed to create thermometer");

    // Wait for initial data
    thread::sleep(Duration::from_millis(300));

    // Create home
    let home = create_home!(
        "Smart Home",
        {
            "Living Room",
            create_room!(
                "Living Room",
                "TV Outlet" => Device::from(outlet)
            )
        },
        {
            "Kitchen",
            create_room!(
                "Kitchen",
                "Thermometer" => Device::from(thermometer)
            )
        }
    );

    // Display status
    println!("{}", home.info());

    // Simulators automatically cleaned up when spawner drops
}
```

### Example 3: Direct Device Control

```rust
use smart_home::smart_devices::{OutletDevice, TemperatureSensor};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::simulators::outlet::{OutletSimulator, OutletSimulatorConfig};

fn main() {
    // Spawn simulator
    let config = OutletSimulatorConfig::new("127.0.0.1:0", 100);
    let simulator = OutletSimulator::spawn(config).unwrap();

    // Connect remote outlet
    let mut outlet = OutletRemote::new(
        "Test Outlet".to_string(),
        simulator.address().to_string(),
    ).unwrap();

    // Control the outlet
    outlet.turn_on().unwrap();
    assert_eq!(outlet.state().unwrap(), OutletState::On);
    assert_eq!(outlet.power_usage().unwrap(), 100);

    outlet.turn_off().unwrap();
    assert_eq!(outlet.power_usage().unwrap(), 0);
}
```

---

## 🧪 Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suites

```bash
# Test spawner functionality
cargo test spawner

# Test smart home hierarchy
cargo test smart_home

# Test remote devices
cargo test remote
```

### Test Coverage

- **Unit Tests**: Device implementations, protocol encoding/decoding
- **Integration Tests**: Full stack with simulators
  - Outlet simulator with remote connections
  - Thermometer simulator with UDP communication
  - Multiple concurrent devices
  - Smart home hierarchy with remote devices

Current test count: **119 passing tests**

---

## 🔧 Development

### Code Quality

```bash
# Run clippy (no warnings allowed)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Architecture Documentation

Detailed technical documentation is available in:
- `.claude/architecture.md` - Complete implementation guide
- `.claude/SIMULATOR_ARCHITECTURE.md` - Simulator design patterns
- `.claude/review.md` - Recent code review

---

## 🌟 Key Design Decisions

### 1. Config Object Pattern
Simulators use configuration objects instead of individual parameters, providing:
- Better API ergonomics
- Easy extensibility
- Type-safe configuration
- Builder pattern support

### 2. Owned Address Returns
Spawner methods return `String` addresses instead of references, solving:
- Borrow checker conflicts with multiple spawns
- Cleaner API for sequential operations
- No scope management needed

### 3. Library-Based Simulators
Simulators are library components, not just binaries:
- Easy programmatic testing
- Single-process integration tests
- Automatic cleanup via `Drop`
- No external process management

### 4. Thread Safety
- `Arc<Mutex<>>` for shared state in TCP server
- `Arc<AtomicF32>` for thermometer temperature
- Background threads with clean shutdown
- No unsafe code

### 5. Error Handling
- Result types for all fallible operations
- Custom error types per device
- Network errors propagated correctly
- Graceful degradation where appropriate

---

## 📚 API Highlights

### Macros

```rust
// Create home with nested structure
create_home!(name, {room_name, room}, ...);

// Create room with devices
create_room!(name, device_name => device, ...);
```

### Traits

```rust
// Common information trait
pub trait Information {
    fn info(&self) -> String;
}

// Outlet device trait
pub trait OutletDevice {
    fn turn_on(&mut self) -> Result<(), DeviceError>;
    fn turn_off(&mut self) -> Result<(), DeviceError>;
    fn switch(&mut self) -> Result<(), DeviceError>;
    fn state(&self) -> Result<OutletState, DeviceError>;
    fn power_usage(&self) -> Result<Watt, DeviceError>;
}

// Temperature sensor trait
pub trait TemperatureSensor {
    fn current_temperature(&self) -> Celsius;
}
```

---

## 🎯 Homework Requirements

This project fulfills all homework requirements:

✅ Smart outlet with TCP communication and simulation mode
✅ Smart thermometer with UDP reception in parallel thread
✅ Non-blocking outlet simulator supporting multiple clients
✅ Non-blocking thermometer simulator with configurable intervals
✅ Example application demonstrating full system
✅ Comprehensive error handling and reporting
✅ Unit and integration tests
✅ Clean clippy and fmt checks

---

## 📄 License

This is a homework project for OTUS Rust Developer course.

---

## 🤝 Contributing

This is an educational project. Feel free to explore and learn from the code!

### Future Enhancements

- [ ] Device discovery protocol (mDNS/Zeroconf)
- [ ] Authentication and encryption (TLS/DTLS)
- [ ] Persistence layer for home configuration
- [ ] Web API for remote management
- [ ] More device types (lights, locks, cameras)
- [ ] Event notification system
- [ ] Historical data logging

---

## 📞 Support

For questions or issues related to this project, please refer to:
- Architecture documentation in `.claude/` directory
- Inline code documentation
- Test examples for usage patterns
