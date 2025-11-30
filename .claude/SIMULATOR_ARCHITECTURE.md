# Device Simulator Architecture

## Overview

This document describes the library-based device simulator architecture for the Smart Home project. The simulators provide an easy-to-use API for spawning device simulators programmatically, making testing and integration much simpler.

---

## Architecture Changes

### Old Approach (Binary-Only)

**Problems:**
- Required running separate terminal windows
- Manual process management
- Difficult to test programmatically
- Complex setup in CI/CD
- No automatic cleanup
- Required shell scripts to coordinate

**Structure:**
```
src/bin/outlet_simulator.rs       (standalone binary)
src/bin/thermometer_simulator.rs  (standalone binary)
```

**Usage:**
```bash
# Terminal 1
cargo run --bin outlet_simulator 127.0.0.1:8001 150

# Terminal 2
cargo run --bin thermometer_simulator config.toml

# Terminal 3
cargo run --example my_app
```

### New Approach (Library-Based with Optional Binaries)

**Advantages:**
- ✅ Simple spawn API: `OutletSimulator::spawn(config)`
- ✅ Automatic cleanup via `Drop` trait
- ✅ Easy integration testing
- ✅ No separate processes to manage
- ✅ Perfect for CI/CD pipelines
- ✅ Great developer experience
- ✅ Still supports standalone binaries if needed

**Structure:**
```
src/simulators/
  ├── mod.rs                  (module exports)
  ├── outlet.rs               (OutletSimulator library)
  └── thermometer.rs          (ThermometerSimulator library)

src/bin/                      (optional binaries)
  ├── outlet_simulator.rs     (thin wrapper)
  └── thermometer_simulator.rs (thin wrapper)
```

**Usage:**
```rust
// Everything in one process!
let sim = OutletSimulator::spawn(config)?;
let outlet = OutletRemote::new("name", sim.addr())?;

outlet.turn_on()?;
// ... use the device

// Automatic cleanup when sim is dropped
```

---

## API Design

### Outlet Simulator

**Configuration:**
```rust
pub struct OutletSimulatorConfig {
    pub tcp_addr: String,      // e.g., "127.0.0.1:8001" or "127.0.0.1:0"
    pub power_watts: u32,      // Power consumption when ON
}

impl OutletSimulatorConfig {
    pub fn new(tcp_addr: impl Into<String>, power_watts: u32) -> Self;
}
```

**Simulator Handle:**
```rust
pub struct OutletSimulator {
    // Internal fields
}

impl OutletSimulator {
    /// Spawn a new outlet simulator
    pub fn spawn(config: OutletSimulatorConfig) -> io::Result<Self>;

    /// Get the actual bound address (useful when binding to port 0)
    pub fn addr(&self) -> &str;
}

impl Drop for OutletSimulator {
    // Automatic cleanup
}
```

**Example:**
```rust
use smart_home::simulators::{OutletSimulator, OutletSimulatorConfig};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::OutletDevice;

// Spawn simulator on random port
let config = OutletSimulatorConfig::new("127.0.0.1:0", 150);
let simulator = OutletSimulator::spawn(config)?;

// Connect to it
let mut outlet = OutletRemote::new(
    "My Outlet".to_string(),
    simulator.addr().to_string(),
)?;

// Use it
outlet.turn_on()?;
assert_eq!(outlet.power_usage()?, 150);

// Automatic cleanup when simulator drops
```

### Thermometer Simulator

**Temperature Patterns:**
```rust
pub enum TemperaturePattern {
    RandomWalk {
        min: f32,
        max: f32,
        step: f32,
    },
    SineWave {
        center: f32,
        amplitude: f32,
        period_secs: f32,
    },
    Constant(f32),
}
```

**Configuration:**
```rust
pub struct ThermometerSimulatorConfig {
    pub target_addr: String,          // Where to send UDP packets
    pub send_interval: Duration,      // How often to send
    pub pattern: TemperaturePattern,  // Temperature generation pattern
    pub initial_temp: Option<f32>,    // Optional initial value
}

impl ThermometerSimulatorConfig {
    pub fn new(target_addr: impl Into<String>, send_interval: Duration) -> Self;
    pub fn with_pattern(self, pattern: TemperaturePattern) -> Self;
    pub fn with_initial_temp(self, temp: f32) -> Self;
}
```

**Simulator Handle:**
```rust
pub struct ThermometerSimulator {
    // Internal fields
}

impl ThermometerSimulator {
    /// Spawn a new thermometer simulator
    pub fn spawn(config: ThermometerSimulatorConfig) -> io::Result<Self>;

    /// Gracefully stop the simulator
    pub fn stop(&self);
}

impl Drop for ThermometerSimulator {
    // Automatic cleanup and thread join
}
```

**Example:**
```rust
use smart_home::simulators::{
    ThermometerSimulator,
    ThermometerSimulatorConfig,
    TemperaturePattern,
};
use smart_home::smart_devices::thermometr_remote::ThermometerRemote;
use std::time::Duration;

// Create receiver first
let receiver = ThermometerRemote::new(
    "My Thermometer".to_string(),
    "127.0.0.1:9001".to_string(),
)?;

// Spawn simulator with random walk pattern
let pattern = TemperaturePattern::RandomWalk {
    min: 20.0,
    max: 25.0,
    step: 0.3,
};

let config = ThermometerSimulatorConfig::new(
    "127.0.0.1:9001",
    Duration::from_millis(500),
)
.with_pattern(pattern)
.with_initial_temp(22.5);

let simulator = ThermometerSimulator::spawn(config)?;

// Read temperatures
thread::sleep(Duration::from_secs(1));
let temp = receiver.current_temperature();
println!("Temperature: {:.2}°C", temp);

// Automatic cleanup when simulator drops
```

---

## Complete Integration Example

```rust
use smart_home::create_home;
use smart_home::create_room;
use smart_home::simulators::{
    OutletSimulator, OutletSimulatorConfig,
    ThermometerSimulator, ThermometerSimulatorConfig,
    TemperaturePattern,
};
use smart_home::smart_devices::{Device, OutletDevice, TemperatureSensor};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::thermometr_remote::ThermometerRemote;
use std::thread;
use std::time::Duration;

fn main() {
    // 1. Spawn simulators
    let outlet_config = OutletSimulatorConfig::new("127.0.0.1:0", 150);
    let outlet_sim = OutletSimulator::spawn(outlet_config)
        .expect("Failed to spawn outlet simulator");

    let thermo_config = ThermometerSimulatorConfig::new(
        "127.0.0.1:9001",
        Duration::from_millis(500),
    ).with_pattern(TemperaturePattern::RandomWalk {
        min: 20.0,
        max: 24.0,
        step: 0.3,
    });

    let thermo_sim = ThermometerSimulator::spawn(thermo_config)
        .expect("Failed to spawn thermometer simulator");

    thread::sleep(Duration::from_millis(100));

    // 2. Create remote devices
    let outlet = OutletRemote::new(
        "Living Room Outlet".to_string(),
        outlet_sim.addr().to_string(),
    ).expect("Failed to connect to outlet");

    let thermometer = ThermometerRemote::new(
        "Living Room Thermometer".to_string(),
        "127.0.0.1:9001".to_string(),
    ).expect("Failed to create thermometer");

    // 3. Create smart home
    let home = create_home!(
        "My Smart Home",
        {
            "Living Room",
            create_room!(
                "Living Room",
                "outlet" => Device::OutletTypeRemote(outlet),
                "thermometer" => Device::ThermometerTypeRemote(thermometer)
            )
        }
    );

    // 4. Use the devices
    println!("{}", home.info());

    // Everything automatically cleaned up when dropped!
}
```

---

## Migration Guide

### For Existing Code

If you have existing code that uses separate binaries:

**Before:**
```bash
# Terminal 1
cargo run --bin outlet_simulator 127.0.0.1:8001 150

# Terminal 2 - your code
cargo run --example my_app
```

**After:**
```rust
// In your example/main.rs
let config = OutletSimulatorConfig::new("127.0.0.1:8001", 150);
let _simulator = OutletSimulator::spawn(config)?;

// Rest of your code...
```

### For Testing

**Before:**
```rust
#[test]
fn test_outlet() {
    // Had to use mock TCP server or external process
    let listener = TcpListener::bind("127.0.0.1:0")?;
    // Complex mock setup...
}
```

**After:**
```rust
#[test]
fn test_outlet() {
    let config = OutletSimulatorConfig::new("127.0.0.1:0", 150);
    let sim = OutletSimulator::spawn(config).unwrap();

    let mut outlet = OutletRemote::new("Test", sim.addr()).unwrap();
    outlet.turn_on().unwrap();

    assert_eq!(outlet.state().unwrap(), OutletState::On);
    // Automatic cleanup
}
```

---

## Implementation Details

### Thread Management

Both simulators run in background threads:

**Outlet Simulator:**
- Main thread accepts TCP connections
- Spawns a handler thread per client
- Uses non-blocking accept for graceful shutdown
- State shared via `Arc<Mutex<>>`

**Thermometer Simulator:**
- Single thread sends UDP packets
- Uses `AtomicBool` for shutdown signaling
- `Drop` impl waits for thread to finish

### Resource Cleanup

Both simulators implement `Drop` trait:

```rust
impl Drop for OutletSimulator {
    fn drop(&mut self) {
        // Non-blocking mode allows thread to exit naturally
        if let Some(handle) = self.listener_thread.take() {
            drop(handle); // Thread stops when listener closes
        }
    }
}

impl Drop for ThermometerSimulator {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for thread to finish
        if let Some(handle) = self.sender_thread.take() {
            let _ = handle.join();
        }
    }
}
```

---

## Optional Binary Wrappers

For users who still want standalone binaries, thin wrappers are provided:

**src/bin/outlet_simulator.rs:**
```rust
use smart_home::simulators::{OutletSimulator, OutletSimulatorConfig};

fn main() {
    let config = OutletSimulatorConfig::new(
        env::args().nth(1).expect("address"),
        env::args().nth(2).expect("power").parse().expect("number"),
    );

    let simulator = OutletSimulator::spawn(config).expect("Failed to spawn");

    println!("Running on {}", simulator.addr());
    println!("Press Enter to stop...");

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).ok();
}
```

Usage:
```bash
cargo run --bin outlet_simulator 127.0.0.1:8001 150
```

---

## Summary

The new library-based simulator architecture provides:

1. **Simple API** - One function call to spawn a simulator
2. **Automatic cleanup** - No manual resource management
3. **Easy testing** - Perfect for unit and integration tests
4. **Better ergonomics** - Everything in one process
5. **CI/CD friendly** - No complex setup required
6. **Flexible patterns** - Multiple temperature generation modes
7. **Optional binaries** - Can still use as standalone if needed

This architecture makes the smart home system much easier to use, test, and maintain.
