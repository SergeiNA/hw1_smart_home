# Step 5: Create Thermometer Simulator (UDP Sender)

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 3: Add UDP Support to Thermometer](step-3-thermometer-remote.md)

**Goal:** Create a library-based thermometer simulator with a simple spawn API

---

## Overview

The thermometer simulator will:
- Be a library module in `src/simulators/thermometer.rs`
- Provide a simple `ThermometerSimulator::spawn()` API
- Send temperature readings via UDP at regular intervals
- Generate realistic temperature values (random walk, sine wave, etc.)
- Return a handle that can be used to control or stop the simulator
- Can be used from examples, binaries, or tests

### Architecture

```
┌─────────────────────────────┐
│   Example or Binary         │
│                             │
│  let sim =                  │
│    ThermometerSimulator::   │
│      spawn(config)?;        │
│                             │
│  // Simulator sends UDP     │
│  // packets in background   │
│                             │
│  // Use ThermometerRemote   │
│  // to receive data         │
│                             │
│  drop(sim); // Auto cleanup │
└─────────────────────────────┘
```

---

## 5.1 Update Simulator Module

**File:** `src/simulators.rs`

```rust
pub mod outlet;
pub mod thermometer;

pub use outlet::{OutletSimulator, OutletSimulatorConfig};
pub use thermometer::{ThermometerSimulator, ThermometerSimulatorConfig, TemperaturePattern};
```

---

## 5.2 Create Thermometer Simulator Configuration

**File:** `src/simulators/thermometer.rs`

```rust
use crate::protocols::thermometer::TemperatureData;
use std::io;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Temperature generation pattern
#[derive(Debug, Clone, Copy)]
pub enum TemperaturePattern {
    /// Random walk within range
    RandomWalk {
        min: f32,
        max: f32,
        step: f32,
    },
    /// Sine wave pattern
    SineWave {
        center: f32,
        amplitude: f32,
        period_secs: f32,
    },
    /// Fixed constant value
    Constant(f32),
}

impl Default for TemperaturePattern {
    fn default() -> Self {
        TemperaturePattern::RandomWalk {
            min: 18.0,
            max: 26.0,
            step: 0.5,
        }
    }
}

/// Configuration for the thermometer simulator
#[derive(Debug, Clone)]
pub struct ThermometerSimulatorConfig {
    /// Target UDP address to send readings to (e.g., "127.0.0.1:9001")
    pub target_addr: String,
    /// How often to send readings
    pub send_interval: Duration,
    /// Temperature generation pattern
    pub pattern: TemperaturePattern,
    /// Initial temperature (optional, pattern-specific default used if None)
    pub initial_temp: Option<f32>,
}

impl ThermometerSimulatorConfig {
    pub fn new(
        target_addr: impl Into<String>,
        send_interval: Duration,
    ) -> Self {
        Self {
            target_addr: target_addr.into(),
            send_interval,
            pattern: TemperaturePattern::default(),
            initial_temp: None,
        }
    }

    pub fn with_pattern(mut self, pattern: TemperaturePattern) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn with_initial_temp(mut self, temp: f32) -> Self {
        self.initial_temp = Some(temp);
        self
    }
}
```

---

## 5.3 Implement Temperature Generator

```rust
/// Temperature value generator
struct TemperatureGenerator {
    pattern: TemperaturePattern,
    current: f32,
    iteration: u64,
}

impl TemperatureGenerator {
    fn new(pattern: TemperaturePattern, initial: Option<f32>) -> Self {
        let current = initial.unwrap_or_else(|| match pattern {
            TemperaturePattern::RandomWalk { min, max, .. } => (min + max) / 2.0,
            TemperaturePattern::SineWave { center, .. } => center,
            TemperaturePattern::Constant(val) => val,
        });

        Self {
            pattern,
            current,
            iteration: 0,
        }
    }

    fn next(&mut self) -> f32 {
        self.iteration += 1;

        match self.pattern {
            TemperaturePattern::RandomWalk { min, max, step } => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let change = rng.gen_range(-step..=step);
                self.current = (self.current + change).clamp(min, max);
                self.current
            }
            TemperaturePattern::SineWave {
                center,
                amplitude,
                period_secs,
            } => {
                let t = self.iteration as f32 / period_secs;
                center + amplitude * (t * 2.0 * std::f32::consts::PI).sin()
            }
            TemperaturePattern::Constant(val) => val,
        }
    }
}
```

---

## 5.4 Implement Thermometer Simulator with Handle

```rust
/// Handle to a running thermometer simulator
/// When dropped, the simulator is automatically stopped
pub struct ThermometerSimulator {
    sender_thread: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl ThermometerSimulator {
    /// Spawn a new thermometer simulator
    ///
    /// Returns a handle to the simulator that will automatically
    /// clean up when dropped.
    pub fn spawn(config: ThermometerSimulatorConfig) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);

        let sender_thread = thread::spawn(move || {
            Self::run_sender(socket, config, shutdown_clone);
        });

        Ok(Self {
            sender_thread: Some(sender_thread),
            shutdown,
        })
    }

    /// Run the UDP sender loop
    fn run_sender(
        socket: UdpSocket,
        config: ThermometerSimulatorConfig,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut generator = TemperatureGenerator::new(
            config.pattern,
            config.initial_temp,
        );

        let mut packet_count = 0u64;

        while !shutdown.load(Ordering::Relaxed) {
            // Generate next temperature
            let temp = generator.next();

            // Create temperature reading
            let reading = TemperatureData::new(temp);
            let buf = reading.to_bytes();

            // Send UDP packet
            match socket.send_to(&buf, &config.target_addr) {
                Ok(_) => {
                    packet_count += 1;
                }
                Err(e) => {
                    eprintln!("[Thermometer Simulator] Failed to send: {}", e);
                }
            }

            // Wait before next reading
            thread::sleep(config.send_interval);
        }
    }

    /// Gracefully stop the simulator
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
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

## 5.5 Add Dependencies

If not already in `Cargo.toml`, add:

```toml
[dependencies]
rand = "0.8"
```

---

## 5.6 Create Example Using Simulator

**File:** `examples/thermometer_simulator_usage.rs`

```rust
use smart_home::simulators::{
    ThermometerSimulator, ThermometerSimulatorConfig, TemperaturePattern
};
use smart_home::smart_devices::thermometr_remote::ThermometerRemote;
use smart_home::smart_devices::TemperatureSensor;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  Thermometer Simulator Library Example    ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Create thermometer receiver first
    let receiver = ThermometerRemote::new(
        "Test Thermometer".to_string(),
        "127.0.0.1:19999".to_string(),
    ).expect("Failed to create receiver");

    println!("✓ Thermometer receiver listening on 127.0.0.1:19999");

    // Spawn thermometer simulator with random walk pattern
    let pattern = TemperaturePattern::RandomWalk {
        min: 20.0,
        max: 25.0,
        step: 0.3,
    };

    let config = ThermometerSimulatorConfig::new(
        "127.0.0.1:19999",
        Duration::from_millis(500),
    )
    .with_pattern(pattern)
    .with_initial_temp(22.5);

    let simulator = ThermometerSimulator::spawn(config)
        .expect("Failed to spawn simulator");

    println!("✓ Thermometer simulator started\n");
    println!("Reading temperatures (every 1 second for 10 seconds):\n");

    // Read temperature 10 times
    for i in 1..=10 {
        thread::sleep(Duration::from_secs(1));
        let temp = receiver.current_temperature();
        println!("  Reading #{:2}: {:.2}°C", i, temp);
    }

    println!("\n✅ Test completed!");
    println!("\nSimulator will be automatically stopped when dropped.");

    // Explicitly stop (optional, Drop will do this anyway)
    simulator.stop();
}
```

---

## 5.7 Create Example with Multiple Patterns

**File:** `examples/thermometer_patterns.rs`

```rust
use smart_home::simulators::{
    ThermometerSimulator, ThermometerSimulatorConfig, TemperaturePattern
};
use smart_home::smart_devices::thermometr_remote::ThermometerRemote;
use smart_home::smart_devices::TemperatureSensor;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║   Thermometer Pattern Examples             ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Example 1: Random Walk
    println!("1. Random Walk Pattern (20-25°C)");
    test_pattern(
        "127.0.0.1:20001",
        TemperaturePattern::RandomWalk {
            min: 20.0,
            max: 25.0,
            step: 0.5,
        },
    );

    thread::sleep(Duration::from_secs(1));

    // Example 2: Sine Wave
    println!("\n2. Sine Wave Pattern (center: 22°C, amplitude: 3°C)");
    test_pattern(
        "127.0.0.1:20002",
        TemperaturePattern::SineWave {
            center: 22.0,
            amplitude: 3.0,
            period_secs: 10.0,
        },
    );

    thread::sleep(Duration::from_secs(1));

    // Example 3: Constant
    println!("\n3. Constant Temperature (23.5°C)");
    test_pattern(
        "127.0.0.1:20003",
        TemperaturePattern::Constant(23.5),
    );

    println!("\n✅ All patterns tested!");
}

fn test_pattern(addr: &str, pattern: TemperaturePattern) {
    let receiver = ThermometerRemote::new(
        "Test".to_string(),
        addr.to_string(),
    ).expect("Failed to create receiver");

    let config = ThermometerSimulatorConfig::new(addr, Duration::from_millis(200))
        .with_pattern(pattern);

    let _simulator = ThermometerSimulator::spawn(config)
        .expect("Failed to spawn simulator");

    thread::sleep(Duration::from_millis(300));

    print!("   Readings: ");
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(300));
        let temp = receiver.current_temperature();
        print!("{:.2}°C ", temp);
    }
    println!();
}
```

---

## 5.8 Create Binary Wrapper (Optional)

**File:** `src/bin/thermometer_simulator.rs`

```rust
use smart_home::simulators::{
    ThermometerSimulator, ThermometerSimulatorConfig, TemperaturePattern
};
use std::io::{self, Write};
use std::time::Duration;

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <target_address> <interval_ms> [min_temp] [max_temp]", args[0]);
        eprintln!("Example: {} 127.0.0.1:9001 1000 18.0 26.0", args[0]);
        std::process::exit(1);
    }

    let target_addr = &args[1];
    let interval_ms: u64 = args[2]
        .parse()
        .expect("Invalid interval_ms argument");

    let pattern = if args.len() >= 5 {
        let min: f32 = args[3].parse().expect("Invalid min_temp");
        let max: f32 = args[4].parse().expect("Invalid max_temp");
        TemperaturePattern::RandomWalk {
            min,
            max,
            step: 0.5,
        }
    } else {
        TemperaturePattern::default()
    };

    // Create and spawn simulator
    let config = ThermometerSimulatorConfig::new(
        target_addr,
        Duration::from_millis(interval_ms),
    ).with_pattern(pattern);

    let simulator = ThermometerSimulator::spawn(config)
        .expect("Failed to start simulator");

    println!("╔════════════════════════════════════════════╗");
    println!("║  Smart Thermometer Simulator (Binary UDP) ║");
    println!("╚════════════════════════════════════════════╝");
    println!("Target address: {}", target_addr);
    println!("Send interval: {}ms", interval_ms);
    if let TemperaturePattern::RandomWalk { min, max, .. } = pattern {
        println!("Temperature range: {:.1}°C - {:.1}°C", min, max);
    }
    println!("\nPress Enter to stop...\n");

    // Keep running until Enter is pressed
    let stdin = io::stdin();
    let mut line = String::new();
    let _ = stdin.read_line(&mut line);

    println!("\nShutting down...");
    simulator.stop();
}
```

---

## 5.9 Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_devices::thermometr_remote::ThermometerRemote;
    use crate::smart_devices::TemperatureSensor;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_simulator_spawn() {
        let config = ThermometerSimulatorConfig::new(
            "127.0.0.1:29001",
            Duration::from_millis(100),
        );

        let simulator = ThermometerSimulator::spawn(config);
        assert!(simulator.is_ok());
    }

    #[test]
    fn test_simulator_sends_data() {
        let addr = "127.0.0.1:29002";

        let receiver = ThermometerRemote::new(
            "Test".to_string(),
            addr.to_string(),
        ).unwrap();

        let config = ThermometerSimulatorConfig::new(
            addr,
            Duration::from_millis(100),
        ).with_initial_temp(21.5);

        let _simulator = ThermometerSimulator::spawn(config).unwrap();

        // Wait for data
        thread::sleep(Duration::from_millis(300));

        let temp = receiver.current_temperature();
        // Should have received data (not default 0.0)
        assert!(temp > 0.0);
    }

    #[test]
    fn test_constant_pattern() {
        let addr = "127.0.0.1:29003";

        let receiver = ThermometerRemote::new(
            "Test".to_string(),
            addr.to_string(),
        ).unwrap();

        let pattern = TemperaturePattern::Constant(25.0);
        let config = ThermometerSimulatorConfig::new(
            addr,
            Duration::from_millis(100),
        ).with_pattern(pattern);

        let _simulator = ThermometerSimulator::spawn(config).unwrap();

        thread::sleep(Duration::from_millis(300));

        let temp = receiver.current_temperature();
        assert!((temp - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_random_walk_pattern() {
        let addr = "127.0.0.1:29004";

        let receiver = ThermometerRemote::new(
            "Test".to_string(),
            addr.to_string(),
        ).unwrap();

        let pattern = TemperaturePattern::RandomWalk {
            min: 20.0,
            max: 25.0,
            step: 0.5,
        };

        let config = ThermometerSimulatorConfig::new(
            addr,
            Duration::from_millis(50),
        ).with_pattern(pattern);

        let _simulator = ThermometerSimulator::spawn(config).unwrap();

        thread::sleep(Duration::from_millis(300));

        // Read multiple values
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(100));
            let temp = receiver.current_temperature();
            assert!(temp >= 20.0 && temp <= 25.0);
        }
    }

    #[test]
    fn test_simulator_stop() {
        let config = ThermometerSimulatorConfig::new(
            "127.0.0.1:29005",
            Duration::from_millis(100),
        );

        let simulator = ThermometerSimulator::spawn(config).unwrap();
        simulator.stop();

        thread::sleep(Duration::from_millis(200));
        // Should not panic
    }

    #[test]
    fn test_simulator_cleanup() {
        let config = ThermometerSimulatorConfig::new(
            "127.0.0.1:29006",
            Duration::from_millis(100),
        );

        {
            let _simulator = ThermometerSimulator::spawn(config).unwrap();
            // Dropped here
        }

        thread::sleep(Duration::from_millis(200));
        // Should not panic
    }
}
```

---

## 5.10 Run Tests and Examples

1. **Build and test:**
   ```bash
   cargo test thermometer_simulator
   ```

2. **Run the basic example:**
   ```bash
   cargo run --example thermometer_simulator_usage
   ```

3. **Run the patterns example:**
   ```bash
   cargo run --example thermometer_patterns
   ```

4. **Run as binary (optional):**
   ```bash
   cargo run --bin thermometer_simulator 127.0.0.1:9001 1000 20.0 25.0
   ```

---

## Summary

✅ Created `ThermometerSimulator` library in `src/simulators/thermometer.rs`
✅ Simple spawn API: `ThermometerSimulator::spawn(config)`
✅ Returns handle with automatic cleanup via `Drop`
✅ Supports multiple temperature patterns (random walk, sine, constant)
✅ Configurable send interval and initial temperature
✅ Graceful shutdown with `stop()` method
✅ Can be used from examples, binaries, or tests
✅ Optional binary wrapper for standalone use
✅ Comprehensive test coverage

**Advantages over binary-only approach:**
- Easy to use in integration tests
- No need to manage separate processes
- Automatic cleanup when handle is dropped
- Can spawn multiple simulators programmatically
- Better for CI/CD pipelines
- Flexible pattern configuration

**Next Step:** [Step 6: Create Example Application](step-6-example-app.md)
