# Step 3: Add UDP Support to Smart Thermometer

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)

**Goal:** Extend the `Thermometer` struct to receive temperature updates via UDP in a background thread

---

## Overview

The thermometer will support two modes:
- **Local mode**: Current in-memory implementation
- **Remote mode**: Spawns a background thread that listens for UDP packets and updates temperature

Key challenges:
- Thread lifecycle management (start on create, stop on drop)
- Thread-safe shared state (`Arc<Mutex<f32>>`)
- Clean shutdown signaling

---

## 3.1 Define Operation Mode

**File:** `src/smart_devices/thermometer.rs`

```rust
use crate::protocol::thermometer::TemperatureReading;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::net::UdpSocket;

/// Operation mode for the thermometer
#[derive(Debug)]
pub enum ThermometerMode {
    /// Direct in-memory access
    Local,
    /// UDP reception in background thread
    Remote { udp_addr: String },
}
```

---

## 3.2 Modify Thermometer Struct

Update the struct to support background thread:

```rust
pub struct Thermometer {
    name: String,
    mode: ThermometerMode,
    // Shared temperature (for both modes)
    temperature: Arc<Mutex<Celsius>>,
    // Thread management for remote mode
    receiver_thread: Option<thread::JoinHandle<()>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}
```

**Important:** `Thermometer` can no longer derive `Clone` due to thread handles.

---

## 3.3 Update Constructors

```rust
impl Thermometer {
    /// Create a new thermometer in local mode (existing behavior)
    pub fn new(name: String, initial_temperature: Celsius) -> Self {
        Thermometer {
            name,
            mode: ThermometerMode::Local,
            temperature: Arc::new(Mutex::new(initial_temperature)),
            receiver_thread: None,
            shutdown_tx: None,
        }
    }

    /// Create a new thermometer in remote mode (UDP reception)
    pub fn new_remote(name: String, udp_addr: String) -> Result<Self, std::io::Error> {
        let temperature = Arc::new(Mutex::new(20.0)); // Default initial value
        let temp_clone = Arc::clone(&temperature);
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        // Spawn background UDP receiver thread
        let addr_clone = udp_addr.clone();
        let receiver_thread = thread::spawn(move || {
            Self::udp_receiver_loop(addr_clone, temp_clone, shutdown_rx);
        });

        Ok(Thermometer {
            name,
            mode: ThermometerMode::Remote { udp_addr },
            temperature,
            receiver_thread: Some(receiver_thread),
            shutdown_tx: Some(shutdown_tx),
        })
    }
}
```

---

## 3.4 Implement UDP Receiver Loop

Add the background thread function:

```rust
impl Thermometer {
    /// Background thread that receives UDP packets and updates temperature
    fn udp_receiver_loop(
        udp_addr: String,
        temperature: Arc<Mutex<Celsius>>,
        shutdown_rx: mpsc::Receiver<()>,
    ) {
        // Bind UDP socket
        let socket = match UdpSocket::bind(&udp_addr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to bind UDP socket to {}: {}", udp_addr, e);
                return;
            }
        };

        // Set non-blocking mode for periodic shutdown checks
        if let Err(e) = socket.set_nonblocking(true) {
            eprintln!("Failed to set non-blocking: {}", e);
            return;
        }

        println!("Thermometer UDP receiver listening on {}", udp_addr);

        let mut buf = [0u8; 8];

        loop {
            // Check for shutdown signal
            if shutdown_rx.try_recv().is_ok() {
                println!("Thermometer UDP receiver shutting down");
                break;
            }

            // Try to receive UDP packet
            match socket.recv(&mut buf) {
                Ok(8) => {
                    // Parse temperature reading
                    let reading = TemperatureReading::from_bytes(&buf);

                    // Update shared temperature
                    if let Ok(mut temp) = temperature.lock() {
                        *temp = reading.temperature;
                        println!("  [UDP] Received temperature: {:.2}°C at timestamp {}",
                                 reading.temperature, reading.timestamp);
                    }
                }
                Ok(n) => {
                    eprintln!("  [UDP] Invalid packet size: {} bytes (expected 8)", n);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, sleep briefly
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    eprintln!("  [UDP] Receive error: {}", e);
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
}
```

---

## 3.5 Implement Drop for Clean Shutdown

Ensure the thread is properly cleaned up:

```rust
impl Drop for Thermometer {
    fn drop(&mut self) {
        // Signal the thread to shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Wait for thread to finish
        if let Some(handle) = self.receiver_thread.take() {
            if let Err(e) = handle.join() {
                eprintln!("Error joining thermometer thread: {:?}", e);
            }
        }
    }
}
```

---

## 3.6 Update Trait Implementation

The trait implementation remains simple - just read the shared temperature:

```rust
impl TemperatureSensor for Thermometer {
    fn current_temperature(&self) -> Celsius {
        *self.temperature.lock().unwrap()
    }
}
```

---

## 3.7 Update Information Trait

```rust
impl Information for Thermometer {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn info(&self) -> String {
        let temp = self.current_temperature();
        format!(
            "Thermometer: {} - Current Temperature: {:.2}°C",
            self.name, temp
        )
    }
}
```

---

## 3.8 Implement Debug Manually

Since we removed the `Debug` derive:

```rust
impl std::fmt::Debug for Thermometer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Thermometer")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .field("temperature", &self.current_temperature())
            .finish()
    }
}
```

---

## 3.9 Add Device Constructor

**File:** `src/smart_devices.rs`

```rust
impl Device {
    pub fn new_thermometer(name: String, initial_temperature: Celsius) -> Self {
        Device::ThermometerType(Thermometer::new(name, initial_temperature))
    }

    // Add this new constructor
    pub fn new_thermometer_remote(name: String, udp_addr: String) -> Result<Self, std::io::Error> {
        Ok(Device::ThermometerType(Thermometer::new_remote(name, udp_addr)?))
    }
}
```

---

## 3.10 Handle Clone Issue for Device Enum

Since `Thermometer` can't be cloned, you have options:

**Option A: Remove Clone from Device**
```rust
// In src/smart_devices.rs
#[derive(Debug, PartialEq)]  // Remove Clone
pub enum Device {
    OutletType(Outlet),
    ThermometerType(Thermometer),
    Empty,
}
```

**Option B: Implement Clone to create new connections**
```rust
impl Clone for Thermometer {
    fn clone(&self) -> Self {
        match &self.mode {
            ThermometerMode::Local => {
                Thermometer::new(self.name.clone(), self.current_temperature())
            }
            ThermometerMode::Remote { udp_addr } => {
                // Create new UDP receiver with same address
                Thermometer::new_remote(self.name.clone(), udp_addr.clone())
                    .expect("Failed to clone remote thermometer")
            }
        }
    }
}
```

**Option C: Use Arc (Recommended)**
```rust
// Wrap in Arc at room/home level
// Don't clone devices, share them
```

---

## 3.11 Update Existing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermometer_create_test() {
        let thermometer = Thermometer::new("Living Room".to_string(), 22.5 as Celsius);
        assert_eq!(thermometer.name(), "Living Room");
        assert_eq!(thermometer.current_temperature(), 22.5 as Celsius);
        assert_eq!(
            thermometer.info(),
            "Thermometer: Living Room - Current Temperature: 22.50°C"
        );
    }

    #[test]
    fn thermometer_remote_create_test() {
        // Test that remote thermometer can be created
        let result = Thermometer::new_remote(
            "Test Thermometer".to_string(),
            "127.0.0.1:19999".to_string()
        );
        assert!(result.is_ok());

        let thermometer = result.unwrap();
        assert_eq!(thermometer.name(), "Test Thermometer");
        // Initial temperature should be default (20.0)
        assert_eq!(thermometer.current_temperature(), 20.0);
    }
}
```

---

## Verification

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Run tests:**
   ```bash
   cargo test thermometer
   ```

3. **Test UDP reception manually:**

   Terminal 1 - Create a simple UDP sender:
   ```bash
   # Use netcat or create a simple Rust program
   echo -ne '\x00\x00\xb4\x41\x00\x00\x00\x00' | nc -u 127.0.0.1 9001
   ```

   Terminal 2 - Run test program:
   ```rust
   // Create small test binary
   use smart_home::smart_devices::Thermometer;
   use std::thread;
   use std::time::Duration;

   fn main() {
       let thermo = Thermometer::new_remote(
           "Test".to_string(),
           "127.0.0.1:9001".to_string()
       ).unwrap();

       thread::sleep(Duration::from_secs(2));
       println!("Temperature: {}", thermo.current_temperature());
   }
   ```

---

## Common Issues & Solutions

### Issue 1: "Address already in use"

**Problem:** UDP socket already bound

**Solution:**
- Kill existing process
- Use different port
- Add `SO_REUSEADDR` option

### Issue 2: Thread doesn't shut down

**Problem:** Thread blocks on `recv()`

**Solution:** Already handled - we use non-blocking mode

### Issue 3: Clone not available

**Problem:** Device enum can't be cloned

**Solution:** See section 3.10 above

---

## Summary

✅ Added `ThermometerMode` enum for local/remote switching
✅ Created `Thermometer::new_remote()` constructor
✅ Implemented background UDP receiver thread
✅ Added proper thread lifecycle management with `Drop`
✅ Used `Arc<Mutex<Celsius>>` for thread-safe temperature sharing
✅ Added shutdown signaling with `mpsc::channel`
✅ Maintained backward compatibility with local mode

**Next Step:** [Step 4: Create Outlet Simulator](step-4-outlet-simulator.md)
