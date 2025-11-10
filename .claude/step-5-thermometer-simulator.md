# Step 5: Create Thermometer Simulator (UDP Sender)

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 3: Add UDP Support to Thermometer](step-3-thermometer-remote.md)

**Goal:** Create a standalone binary that simulates a physical thermometer by sending temperature readings via UDP

---

## Overview

The thermometer simulator will:
- Read configuration from a TOML file
- Generate realistic temperature values (random walk)
- Send temperature readings via UDP at regular intervals
- Run indefinitely until stopped

---

## 5.1 Create Configuration File Format

**File:** `thermometer_config.toml` (project root)

```toml
[thermometer]
# Target address where temperature readings will be sent
target_address = "127.0.0.1:9001"

# How often to send readings (milliseconds)
send_interval_ms = 1000

# Temperature range for realistic values
min_temperature = 18.0
max_temperature = 26.0
```

---

## 5.2 Add Dependencies

**File:** `Cargo.toml`

Add these dependencies if not already present:

```toml
[dependencies]
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

---

## 5.3 Create Binary File

**File:** `src/bin/thermometer_simulator.rs`

```rust
use smart_home::protocol::thermometer::TemperatureReading;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ThermometerConfig {
    target_address: String,
    send_interval_ms: u64,
    min_temperature: f32,
    max_temperature: f32,
}

fn read_config(path: &str) -> Result<ThermometerConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: ThermometerConfig = toml::from_str(&content)?;
    Ok(config)
}

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "thermometer_config.toml"
    };

    // Read configuration
    let config = read_config(config_path).unwrap_or_else(|e| {
        eprintln!("Failed to read config file '{}': {}", config_path, e);
        eprintln!("\nExpected TOML format:");
        eprintln!("[thermometer]");
        eprintln!("target_address = \"127.0.0.1:9001\"");
        eprintln!("send_interval_ms = 1000");
        eprintln!("min_temperature = 18.0");
        eprintln!("max_temperature = 26.0");
        std::process::exit(1);
    });

    // Validate configuration
    if config.min_temperature >= config.max_temperature {
        eprintln!("Error: min_temperature must be less than max_temperature");
        std::process::exit(1);
    }

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap_or_else(|e| {
        eprintln!("Failed to create UDP socket: {}", e);
        std::process::exit(1);
    });

    // Print startup banner
    print_banner(&config);

    // Run simulation loop
    run_simulation(socket, config);
}

fn print_banner(config: &ThermometerConfig) {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Smart Thermometer Simulator (Binary UDP) ║");
    println!("╚════════════════════════════════════════════╝");
    println!("Target address: {}", config.target_address);
    println!("Send interval: {}ms", config.send_interval_ms);
    println!("Temperature range: {:.1}°C - {:.1}°C\n",
             config.min_temperature, config.max_temperature);
}

fn run_simulation(socket: UdpSocket, config: ThermometerConfig) {
    let mut rng = rand::thread_rng();

    // Start with random temperature in range
    let mut current_temp = rng.gen_range(
        config.min_temperature..config.max_temperature
    );

    let mut packet_count = 0u64;

    loop {
        // Generate realistic temperature change (random walk)
        let change = rng.gen_range(-0.5..0.5);
        current_temp = (current_temp + change)
            .max(config.min_temperature)
            .min(config.max_temperature);

        // Create temperature reading
        let reading = TemperatureReading::new(current_temp);
        let buf = reading.to_bytes();

        // Send UDP packet
        match socket.send_to(&buf, &config.target_address) {
            Ok(_) => {
                packet_count += 1;
                println!("[{}] Packet #{}: {:.2}°C (timestamp: {})",
                         get_timestamp_str(),
                         packet_count,
                         reading.temperature,
                         reading.timestamp);
            }
            Err(e) => {
                eprintln!("[{}] Failed to send: {}",
                         get_timestamp_str(), e);
            }
        }

        // Wait before next reading
        thread::sleep(Duration::from_millis(config.send_interval_ms));
    }
}

fn get_timestamp_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now.to_string()
}
```

---

## 5.4 Create Example Configuration Files

Create different configs for different scenarios:

**File:** `thermometer_living_room.toml`
```toml
[thermometer]
target_address = "127.0.0.1:9001"
send_interval_ms = 1000
min_temperature = 20.0
max_temperature = 24.0
```

**File:** `thermometer_bedroom.toml`
```toml
[thermometer]
target_address = "127.0.0.1:9002"
send_interval_ms = 1500
min_temperature = 18.0
max_temperature = 22.0
```

**File:** `thermometer_kitchen.toml`
```toml
[thermometer]
target_address = "127.0.0.1:9003"
send_interval_ms = 2000
min_temperature = 22.0
max_temperature = 28.0
```

---

## 5.5 Build and Test

1. **Build the simulator:**
   ```bash
   cargo build --bin thermometer_simulator
   ```

2. **Create a test config:**
   ```bash
   cat > test_thermometer.toml << EOF
   [thermometer]
   target_address = "127.0.0.1:9001"
   send_interval_ms = 1000
   min_temperature = 18.0
   max_temperature = 26.0
   EOF
   ```

3. **Run the simulator:**
   ```bash
   cargo run --bin thermometer_simulator test_thermometer.toml
   ```

   You should see:
   ```
   ╔════════════════════════════════════════════╗
   ║  Smart Thermometer Simulator (Binary UDP) ║
   ╚════════════════════════════════════════════╝
   Target address: 127.0.0.1:9001
   Send interval: 1000ms
   Temperature range: 18.0°C - 26.0°C

   [1699876543] Packet #1: 22.34°C (timestamp: 1699876543)
   [1699876544] Packet #2: 22.67°C (timestamp: 1699876544)
   [1699876545] Packet #3: 22.45°C (timestamp: 1699876545)
   ...
   ```

---

## 5.6 Create Test Receiver

Create a test to verify the thermometer receives data:

**File:** `examples/test_thermometer_simulator.rs`

```rust
use smart_home::smart_devices::Thermometer;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Testing thermometer simulator...\n");

    println!("Make sure to start the thermometer simulator first:");
    println!("  cargo run --bin thermometer_simulator test_thermometer.toml\n");

    // Create remote thermometer
    let thermometer = Thermometer::new_remote(
        "Test Thermometer".to_string(),
        "127.0.0.1:9001".to_string()
    ).expect("Failed to create remote thermometer");

    println!("Thermometer created, listening for UDP packets...\n");

    // Read temperature every 2 seconds for 10 iterations
    for i in 1..=10 {
        thread::sleep(Duration::from_secs(2));
        let temp = thermometer.current_temperature();
        println!("Reading #{}: {:.2}°C", i, temp);
    }

    println!("\n✅ Test completed!");
}
```

---

## 5.7 Run Integration Test

**Terminal 1** - Start simulator:
```bash
cargo run --bin thermometer_simulator test_thermometer.toml
```

**Terminal 2** - Run test receiver:
```bash
cargo run --example test_thermometer_simulator
```

Expected output:

**Simulator (Terminal 1):**
```
╔════════════════════════════════════════════╗
║  Smart Thermometer Simulator (Binary UDP) ║
╚════════════════════════════════════════════╝
Target address: 127.0.0.1:9001
Send interval: 1000ms
Temperature range: 18.0°C - 26.0°C

[1699876543] Packet #1: 22.34°C (timestamp: 1699876543)
[1699876544] Packet #2: 22.67°C (timestamp: 1699876544)
[1699876545] Packet #3: 22.45°C (timestamp: 1699876545)
...
```

**Test Client (Terminal 2):**
```
Testing thermometer simulator...

Make sure to start the thermometer simulator first:
  cargo run --bin thermometer_simulator test_thermometer.toml

Thermometer created, listening for UDP packets...

  [UDP] Received temperature: 22.34°C at timestamp 1699876543
Reading #1: 22.34°C
  [UDP] Received temperature: 22.89°C at timestamp 1699876545
Reading #2: 22.89°C
  [UDP] Received temperature: 22.56°C at timestamp 1699876547
Reading #3: 22.56°C
...

✅ Test completed!
```

---

## 5.8 Add Enhanced Features (Optional)

### 5.8.1 Add Graceful Shutdown

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    // ... existing setup ...

    // Set up Ctrl+C handler
    ctrlc::set_handler(|| {
        println!("\n\nReceived shutdown signal...");
        RUNNING.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    run_simulation(socket, config);
}

fn run_simulation(socket: UdpSocket, config: ThermometerConfig) {
    // ... existing setup ...

    while RUNNING.load(Ordering::SeqCst) {
        // ... send temperature ...
        thread::sleep(Duration::from_millis(config.send_interval_ms));
    }

    println!("Simulator shut down gracefully");
}
```

### 5.8.2 Add Temperature Patterns

```rust
#[derive(Deserialize, Debug)]
struct ThermometerConfig {
    target_address: String,
    send_interval_ms: u64,
    min_temperature: f32,
    max_temperature: f32,
    #[serde(default)]
    pattern: Option<String>,  // "random_walk", "sine", "sawtooth"
}

fn generate_temperature(
    config: &ThermometerConfig,
    current: &mut f32,
    iteration: u64
) -> f32 {
    match config.pattern.as_deref() {
        Some("sine") => {
            let t = iteration as f32 * 0.1;
            let mid = (config.min_temperature + config.max_temperature) / 2.0;
            let amp = (config.max_temperature - config.min_temperature) / 2.0;
            mid + amp * t.sin()
        }
        Some("sawtooth") => {
            let range = config.max_temperature - config.min_temperature;
            config.min_temperature + ((iteration as f32 * 0.1) % range)
        }
        _ => {
            // Default: random walk
            let mut rng = rand::thread_rng();
            let change = rng.gen_range(-0.5..0.5);
            *current = (*current + change)
                .max(config.min_temperature)
                .min(config.max_temperature);
            *current
        }
    }
}
```

---

## 5.9 Run Multiple Simulators

You can run multiple thermometer simulators:

```bash
# Terminal 1 - Living Room (fast updates)
cargo run --bin thermometer_simulator thermometer_living_room.toml

# Terminal 2 - Bedroom (medium updates)
cargo run --bin thermometer_simulator thermometer_bedroom.toml

# Terminal 3 - Kitchen (slow updates, warmer)
cargo run --bin thermometer_simulator thermometer_kitchen.toml
```

---

## Summary

✅ Created `thermometer_simulator` binary in `src/bin/`
✅ Implemented TOML configuration file support
✅ Generated realistic temperature values with random walk
✅ Sent binary UDP packets with temperature readings
✅ Added timestamp to each reading
✅ Tested with remote thermometer receiver
✅ Supports multiple concurrent simulators
✅ Optional: Added graceful shutdown and temperature patterns

**Next Step:** [Step 6: Create Example Application](step-6-example-app.md)
