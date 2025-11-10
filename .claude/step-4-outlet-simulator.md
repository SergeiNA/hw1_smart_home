# Step 4: Create Outlet Simulator (TCP Server)

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)

**Goal:** Create a standalone binary that simulates a physical smart outlet by running a TCP server

---

## Overview

The outlet simulator will:
- Listen on a TCP port for incoming connections
- Accept multiple concurrent client connections
- Maintain outlet state (on/off)
- Respond to binary protocol requests
- Run in a separate thread per client

---

## 4.1 Create Binary File

**File:** `src/bin/outlet_simulator.rs`

```rust
use smart_home::protocol::outlet::{OutletRequest, OutletResponse, OutletState};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{self};

/// Simulated outlet device with internal state
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
```

---

## 4.2 Implement Client Handler

```rust
/// Handle a single client connection
fn handle_client(mut stream: TcpStream, outlet: Arc<Mutex<SimulatedOutlet>>) {
    let peer_addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    println!("  [{}] Client connected", peer_addr);

    loop {
        // Receive request
        let request = match OutletRequest::receive(&mut stream) {
            Ok(req) => {
                println!("  [{}] Received: {:?}", peer_addr, req);
                req
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!("  [{}] Client disconnected", peer_addr);
                break;
            }
            Err(e) => {
                eprintln!("  [{}] Error receiving request: {}", peer_addr, e);
                break;
            }
        };

        // Handle request
        let response = {
            let mut outlet = outlet.lock().unwrap();
            outlet.handle_request(request)
        };

        println!("  [{}] Sending: {:?}", peer_addr, response);

        // Send response
        if let Err(e) = response.send(&mut stream) {
            eprintln!("  [{}] Error sending response: {}", peer_addr, e);
            break;
        }
    }

    println!("  [{}] Connection closed", peer_addr);
}
```

---

## 4.3 Implement Main Function

```rust
fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <tcp_address> <power_watts>", args[0]);
        eprintln!("Example: {} 127.0.0.1:8001 150", args[0]);
        std::process::exit(1);
    }

    let addr = &args[1];
    let power_watts: u32 = args[2].parse()
        .expect("Invalid power_watts argument (must be a number)");

    // Create shared outlet state
    let outlet = Arc::new(Mutex::new(SimulatedOutlet::new(power_watts)));

    // Bind TCP listener
    let listener = TcpListener::bind(addr)
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

    // Print startup banner
    print_banner(addr, power_watts);

    // Accept connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let outlet_clone = Arc::clone(&outlet);
                // Spawn a new thread for each client
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

fn print_banner(addr: &str, power_watts: u32) {
    println!("╔════════════════════════════════════════════╗");
    println!("║   Smart Outlet Simulator (Binary TCP)     ║");
    println!("╚════════════════════════════════════════════╝");
    println!("Listening on: {}", addr);
    println!("Power rating: {} watts", power_watts);
    println!("Waiting for connections...\n");
}
```

---

## 4.4 Build and Test

1. **Build the simulator:**
   ```bash
   cargo build --bin outlet_simulator
   ```

2. **Run the simulator:**
   ```bash
   cargo run --bin outlet_simulator 127.0.0.1:8001 150
   ```

   You should see:
   ```
   ╔════════════════════════════════════════════╗
   ║   Smart Outlet Simulator (Binary TCP)     ║
   ╚════════════════════════════════════════════╝
   Listening on: 127.0.0.1:8001
   Power rating: 150 watts
   Waiting for connections...
   ```

---

## 4.5 Create a Test Client

Create a simple test to verify the simulator works:

**File:** `examples/test_outlet_simulator.rs`

```rust
use smart_home::smart_devices::Device;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Testing outlet simulator...\n");

    // Create remote outlet
    let mut outlet = match Device::new_outlet_remote(
        "Test Outlet".to_string(),
        "127.0.0.1:8001".to_string()
    ) {
        Device::OutletType(o) => o,
        _ => panic!("Expected outlet"),
    };

    println!("1. Getting initial state...");
    match outlet.state() {
        Ok(state) => println!("   State: {:?}", state),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n2. Getting power usage...");
    match outlet.power_usage() {
        Ok(power) => println!("   Power: {} watts", power),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n3. Turning on...");
    outlet.turn_on().expect("Failed to turn on");
    thread::sleep(Duration::from_millis(500));

    println!("\n4. Getting state after turn on...");
    match outlet.state() {
        Ok(state) => println!("   State: {:?}", state),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n5. Getting power usage when on...");
    match outlet.power_usage() {
        Ok(power) => println!("   Power: {} watts", power),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n6. Switching state...");
    outlet.switch().expect("Failed to switch");
    thread::sleep(Duration::from_millis(500));

    println!("\n7. Getting final state...");
    match outlet.state() {
        Ok(state) => println!("   State: {:?}", state),
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n✅ Test completed!");
}
```

---

## 4.6 Run Integration Test

Terminal 1 - Start simulator:
```bash
cargo run --bin outlet_simulator 127.0.0.1:8001 150
```

Terminal 2 - Run test client:
```bash
cargo run --example test_outlet_simulator
```

Expected output:

**Simulator:**
```
╔════════════════════════════════════════════╗
║   Smart Outlet Simulator (Binary TCP)     ║
╚════════════════════════════════════════════╝
Listening on: 127.0.0.1:8001
Power rating: 150 watts
Waiting for connections...

  [127.0.0.1:xxxxx] Client connected
  [127.0.0.1:xxxxx] Received: GetState
  [127.0.0.1:xxxxx] Sending: State(Off)
  [127.0.0.1:xxxxx] Received: GetPower
  [127.0.0.1:xxxxx] Sending: Power(0)
  [127.0.0.1:xxxxx] Received: TurnOn
  → Outlet turned ON
  [127.0.0.1:xxxxx] Sending: Ok
  ...
```

**Client:**
```
Testing outlet simulator...

1. Getting initial state...
   State: Off

2. Getting power usage...
   Power: 0 watts

3. Turning on...

4. Getting state after turn on...
   State: On

5. Getting power usage when on...
   Power: 150 watts

6. Switching state...

7. Getting final state...
   State: Off

✅ Test completed!
```

---

## 4.7 Add Multiple Simulators

You can run multiple simulators on different ports:

```bash
# Terminal 1 - Living Room Outlet (150W)
cargo run --bin outlet_simulator 127.0.0.1:8001 150

# Terminal 2 - Kitchen Outlet (300W)
cargo run --bin outlet_simulator 127.0.0.1:8002 300

# Terminal 3 - Bedroom Outlet (100W)
cargo run --bin outlet_simulator 127.0.0.1:8003 100
```

---

## 4.8 Add Graceful Shutdown (Optional Enhancement)

Add signal handling for clean shutdown:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    // ... existing setup ...

    // Set up Ctrl+C handler
    ctrlc::set_handler(|| {
        println!("\n\nReceived shutdown signal...");
        RUNNING.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    // Set listener to non-blocking
    listener.set_nonblocking(true)
        .expect("Failed to set non-blocking");

    // Accept connections with periodic checks
    while RUNNING.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let outlet_clone = Arc::clone(&outlet);
                thread::spawn(move || {
                    handle_client(stream, outlet_clone);
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    println!("Simulator shut down gracefully");
}
```

Add dependency in `Cargo.toml`:
```toml
[dependencies]
ctrlc = "3.4"
```

---

## Summary

✅ Created `outlet_simulator` binary in `src/bin/`
✅ Implemented simulated outlet with internal state
✅ Created TCP server with thread-per-client model
✅ Used binary protocol for type-safe communication
✅ Shared state with `Arc<Mutex<>>`
✅ Tested with example client
✅ Supports multiple concurrent connections
✅ Can run multiple simulators on different ports

**Next Step:** [Step 5: Create Thermometer Simulator](step-5-thermometer-simulator.md)
