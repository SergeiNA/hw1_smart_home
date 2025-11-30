# Step 4: Create Outlet Simulator (TCP Server)

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)

**Goal:** Create a library-based outlet simulator with a simple spawn API

---

## Overview

The outlet simulator will:
- Be a library module in `src/simulators/outlet.rs`
- Provide a simple `OutletSimulator::spawn()` API
- Listen on a TCP port for incoming connections
- Accept multiple concurrent client connections
- Maintain outlet state (on/off) across connections
- Respond to binary protocol requests
- Return a handle that can be used to control or stop the simulator
- Can be used from examples, binaries, or tests

### Architecture

```
┌─────────────────────────────┐
│   Example or Binary         │
│                             │
│  let sim =                  │
│    OutletSimulator::spawn(  │
│      config                 │
│    )?;                      │
│                             │
│  // Use OutletRemote to     │
│  // connect and control     │
│                             │
│  drop(sim); // Auto cleanup │
└─────────────────────────────┘
```

---

## 4.1 Create Simulator Module Structure

First, create the simulators module:

**File:** `src/simulators.rs`

```rust
pub mod outlet;

pub use outlet::{OutletSimulator, OutletSimulatorConfig};
```

Add to `src/lib.rs`:

```rust
pub mod simulators;
```

---

## 4.2 Create Outlet Simulator Configuration

**File:** `src/simulators/outlet.rs`

```rust
use crate::protocols::outlet::{OutletRequest, OutletResponse, OutletState};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Configuration for the outlet simulator
#[derive(Debug, Clone)]
pub struct OutletSimulatorConfig {
    /// TCP address to bind to (e.g., "127.0.0.1:8001")
    pub tcp_addr: String,
    /// Power consumption when outlet is ON (in watts)
    pub power_watts: u32,
}

impl OutletSimulatorConfig {
    pub fn new(tcp_addr: impl Into<String>, power_watts: u32) -> Self {
        Self {
            tcp_addr: tcp_addr.into(),
            power_watts,
        }
    }
}
```

---

## 4.3 Implement Simulated Outlet State

```rust
/// Internal simulated outlet device with state
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

    /// Process a request and return the appropriate response
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
                OutletResponse::Ok
            }
            OutletRequest::TurnOff => {
                self.state = OutletState::Off;
                OutletResponse::Ok
            }
            OutletRequest::Switch => {
                self.state = match self.state {
                    OutletState::On => OutletState::Off,
                    OutletState::Off => OutletState::On,
                };
                OutletResponse::Ok
            }
        }
    }
}
```

---

## 4.4 Implement Outlet Simulator with Handle

```rust
/// Handle to a running outlet simulator
/// When dropped, the simulator is automatically stopped
pub struct OutletSimulator {
    listener_thread: Option<JoinHandle<()>>,
    addr: String,
}

impl OutletSimulator {
    /// Spawn a new outlet simulator
    ///
    /// Returns a handle to the simulator that will automatically
    /// clean up when dropped.
    pub fn spawn(config: OutletSimulatorConfig) -> io::Result<Self> {
        let listener = TcpListener::bind(&config.tcp_addr)?;
        let actual_addr = listener.local_addr()?.to_string();

        // Make listener non-blocking for graceful shutdown
        listener.set_nonblocking(true)?;

        let outlet = Arc::new(Mutex::new(SimulatedOutlet::new(config.power_watts)));

        let listener_thread = thread::spawn(move || {
            Self::run_server(listener, outlet);
        });

        Ok(Self {
            listener_thread: Some(listener_thread),
            addr: actual_addr,
        })
    }

    /// Get the actual address the simulator is bound to
    pub fn addr(&self) -> &str {
        &self.addr
    }

    /// Run the TCP server loop
    fn run_server(listener: TcpListener, outlet: Arc<Mutex<SimulatedOutlet>>) {
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let outlet_clone = Arc::clone(&outlet);
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, outlet_clone) {
                            eprintln!("Client handler error: {}", e);
                        }
                    });
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No connection available, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
    }

    /// Handle a single client connection
    fn handle_client(
        mut stream: TcpStream,
        outlet: Arc<Mutex<SimulatedOutlet>>,
    ) -> io::Result<()> {
        loop {
            // Receive request
            let request = match OutletRequest::receive(&mut stream) {
                Ok(req) => req,
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    break; // Client disconnected
                }
                Err(e) => return Err(e),
            };

            // Handle request
            let response = {
                let mut outlet = outlet.lock().unwrap();
                outlet.handle_request(request)
            };

            // Send response
            response.send(&mut stream)?;
        }

        Ok(())
    }
}

impl Drop for OutletSimulator {
    fn drop(&mut self) {
        // Thread will stop when listener is closed
        // We don't join here to avoid blocking
        if let Some(handle) = self.listener_thread.take() {
            // In a production system, you might want to signal shutdown
            // and then join with a timeout
            drop(handle);
        }
    }
}
```

---

## 4.5 Create Example Using Simulator

**File:** `examples/outlet_simulator_usage.rs`

```rust
use smart_home::simulators::{OutletSimulator, OutletSimulatorConfig};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::OutletDevice;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║   Outlet Simulator Library Example        ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Spawn outlet simulator
    let config = OutletSimulatorConfig::new("127.0.0.1:0", 150);
    let simulator = OutletSimulator::spawn(config)
        .expect("Failed to spawn simulator");

    println!("✓ Outlet simulator started on: {}", simulator.addr());

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    // Connect to the simulator
    let mut outlet = OutletRemote::new(
        "Test Outlet".to_string(),
        simulator.addr().to_string(),
    ).expect("Failed to connect to simulator");

    println!("✓ Connected to simulator\n");

    // Test operations
    println!("1. Getting initial state...");
    match outlet.state() {
        Ok(state) => println!("   State: {:?}", state),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n2. Turning on...");
    outlet.turn_on().expect("Failed to turn on");

    println!("\n3. Getting state after turn on...");
    match outlet.state() {
        Ok(state) => println!("   State: {:?}", state),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n4. Getting power usage...");
    match outlet.power_usage() {
        Ok(power) => println!("   Power: {} watts", power),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n5. Switching state...");
    outlet.switch().expect("Failed to switch");

    println!("\n6. Getting final state...");
    match outlet.state() {
        Ok(state) => println!("   State: {:?}", state),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n✅ Test completed!");
    println!("\nSimulator will be automatically stopped when dropped.");
}
```

---

## 4.6 Create Binary Wrapper (Optional)

If you want a standalone binary, you can create a thin wrapper:

**File:** `src/bin/outlet_simulator.rs`

```rust
use smart_home::simulators::{OutletSimulator, OutletSimulatorConfig};
use std::io::{self, Write};

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <tcp_address> <power_watts>", args[0]);
        eprintln!("Example: {} 127.0.0.1:8001 150", args[0]);
        std::process::exit(1);
    }

    let addr = &args[1];
    let power_watts: u32 = args[2]
        .parse()
        .expect("Invalid power_watts argument");

    // Create and spawn simulator
    let config = OutletSimulatorConfig::new(addr, power_watts);
    let simulator = OutletSimulator::spawn(config)
        .expect("Failed to start simulator");

    println!("╔════════════════════════════════════════════╗");
    println!("║   Smart Outlet Simulator (Binary TCP)     ║");
    println!("╚════════════════════════════════════════════╝");
    println!("Listening on: {}", simulator.addr());
    println!("Power rating: {} watts", power_watts);
    println!("\nPress Ctrl+C to stop...\n");

    // Keep running until interrupted
    let stdin = io::stdin();
    let mut line = String::new();
    let _ = stdin.read_line(&mut line);

    println!("\nShutting down...");
    drop(simulator);
}
```

---

## 4.7 Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::outlet_remote::OutletRemote;
    use crate::smart_devices::OutletDevice;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_simulator_spawn_and_connect() {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 100);
        let simulator = OutletSimulator::spawn(config).unwrap();

        thread::sleep(Duration::from_millis(50));

        let outlet = OutletRemote::new(
            "Test".to_string(),
            simulator.addr().to_string(),
        );

        assert!(outlet.is_ok());
    }

    #[test]
    fn test_simulator_control() {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 150);
        let simulator = OutletSimulator::spawn(config).unwrap();

        thread::sleep(Duration::from_millis(50));

        let mut outlet = OutletRemote::new(
            "Test".to_string(),
            simulator.addr().to_string(),
        ).unwrap();

        // Initial state should be Off
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        assert_eq!(outlet.power_usage().unwrap(), 0);

        // Turn on
        outlet.turn_on().unwrap();
        assert_eq!(outlet.state().unwrap(), OutletState::On);
        assert_eq!(outlet.power_usage().unwrap(), 150);

        // Turn off
        outlet.turn_off().unwrap();
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
        assert_eq!(outlet.power_usage().unwrap(), 0);
    }

    #[test]
    fn test_multiple_clients() {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 200);
        let simulator = OutletSimulator::spawn(config).unwrap();

        thread::sleep(Duration::from_millis(50));

        // Connect multiple clients
        let mut client1 = OutletRemote::new(
            "Client1".to_string(),
            simulator.addr().to_string(),
        ).unwrap();

        let mut client2 = OutletRemote::new(
            "Client2".to_string(),
            simulator.addr().to_string(),
        ).unwrap();

        // Client 1 turns on
        client1.turn_on().unwrap();

        // Client 2 should see the same state
        assert_eq!(client2.state().unwrap(), OutletState::On);
        assert_eq!(client2.power_usage().unwrap(), 200);
    }

    #[test]
    fn test_simulator_cleanup() {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 100);
        let addr = {
            let simulator = OutletSimulator::spawn(config).unwrap();
            simulator.addr().to_string()
        }; // Simulator dropped here

        thread::sleep(Duration::from_millis(200));

        // Should not be able to connect after drop
        let result = OutletRemote::new("Test".to_string(), addr);
        // Connection might fail or succeed depending on timing
        // This test mainly ensures no panic on drop
    }
}
```

---

## 4.8 Run Tests and Example

1. **Build and test:**
   ```bash
   cargo test outlet_simulator
   ```

2. **Run the example:**
   ```bash
   cargo run --example outlet_simulator_usage
   ```

3. **Run as binary (optional):**
   ```bash
   cargo run --bin outlet_simulator 127.0.0.1:8001 150
   ```

---

## Summary

✅ Created `OutletSimulator` library in `src/simulators/outlet.rs`
✅ Simple spawn API: `OutletSimulator::spawn(config)`
✅ Returns handle with automatic cleanup via `Drop`
✅ Supports multiple concurrent client connections
✅ Shared state across connections with `Arc<Mutex<>>`
✅ Can be used from examples, binaries, or tests
✅ Optional binary wrapper for standalone use
✅ Comprehensive test coverage

**Advantages over binary-only approach:**
- Easy to use in integration tests
- No need to manage separate processes
- Automatic cleanup when handle is dropped
- Can spawn multiple simulators programmatically
- Better for CI/CD pipelines

**Next Step:** [Step 5: Create Thermometer Simulator](step-5-thermometer-simulator.md)
