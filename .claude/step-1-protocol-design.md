# Step 1: Protocol Design

**Prerequisites:** None - this is the first step

**Goal:** Define binary communication protocols for TCP (outlet) and UDP (thermometer)

---

## Overview

Before implementing the remote device functionality, we need to define the communication protocols. We'll use:
- **Binary TCP protocol** with Rust enums for type-safe outlet control
- **Binary UDP protocol** with fixed-size structs for temperature readings

---

## 1.1 Create Protocol Module Structure

Create the protocol module directory:

```bash
mkdir -p src/protocol
```

---

## 1.2 Protocol Module Entry Point

**File:** `src/protocol/mod.rs`

```rust
pub mod outlet;
pub mod thermometer;

pub use outlet::{OutletRequest, OutletResponse, OutletState};
pub use thermometer::TemperatureReading;
```

---

## 1.3 TCP Protocol for Smart Outlet

**File:** `src/protocol/outlet.rs`

### Protocol Design

**Message Structure:**
```
[0-3]: u32 length (little-endian)
[4..]: bincode-encoded enum data
```

**Implementation:**

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

        // Validate length to prevent DOS
        if len > 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message too large"
            ));
        }

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

        if len > 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message too large"
            ));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        bincode::deserialize(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
```

### Wire Format Examples

```
Request bytes (bincode encoding):
  GetState:  [0x05, 0x00, 0x00, 0x00] [0x00, 0x00, 0x00, 0x00, 0x00]
             ^^^^^^^^^^^^^^^^^^^^       ^^^^^^^^^^^^^^^^^^^^^^^^
             length = 5                 bincode data

Response bytes:
  State(On):  [length_prefix] [variant_tag, state_data]
  Power(150): [length_prefix] [variant_tag, 0x96, 0x00, 0x00, 0x00]
  Ok:         [length_prefix] [variant_tag]
  Error(msg): [length_prefix] [variant_tag, string_len, ...bytes...]
```

---

## 1.4 UDP Protocol for Smart Thermometer

**File:** `src/protocol/thermometer.rs`

### Protocol Design

**Message Structure (Fixed 8 bytes):**
```
[0-3]: f32 temperature in Celsius (little-endian)
[4-7]: u32 timestamp (Unix epoch, little-endian)
```

**Implementation:**

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TemperatureReading {
    pub temperature: f32,
    pub timestamp: u32,
}

impl TemperatureReading {
    /// Create a new reading with current timestamp
    pub fn new(temperature: f32) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        Self { temperature, timestamp }
    }

    /// Serialize to 8-byte array
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.temperature.to_le_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_le_bytes());
        buf
    }

    /// Deserialize from 8-byte array
    pub fn from_bytes(buf: &[u8; 8]) -> Self {
        let temperature = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let timestamp = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        Self { temperature, timestamp }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_reading_serialization() {
        let reading = TemperatureReading {
            temperature: 22.5,
            timestamp: 1234567890,
        };

        let bytes = reading.to_bytes();
        let decoded = TemperatureReading::from_bytes(&bytes);

        assert_eq!(decoded.temperature, 22.5);
        assert_eq!(decoded.timestamp, 1234567890);
    }

    #[test]
    fn test_temperature_reading_new() {
        let reading = TemperatureReading::new(25.0);
        assert_eq!(reading.temperature, 25.0);
        assert!(reading.timestamp > 0);
    }
}
```

---

## 1.5 Update Dependencies

**File:** `Cargo.toml`

Add the following dependencies:

```toml
[dependencies]
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

---

## 1.6 Update Library Entry Point

**File:** `src/lib.rs`

Add the protocol module:

```rust
pub mod protocol;
pub mod report;
pub mod smart_devices;
pub mod smart_home;
pub mod smart_room;
pub mod traits;
```

---

## Verification

Build the project to ensure protocols compile:

```bash
cargo build
```

You should see:
```
   Compiling smart_home v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

Run protocol tests:

```bash
cargo test protocol
```

---

## Summary

✅ Created `src/protocol/` module structure
✅ Defined binary TCP protocol with `OutletRequest`/`OutletResponse` enums
✅ Defined binary UDP protocol with `TemperatureReading` struct
✅ Added serialization/deserialization methods with length prefixing
✅ Added validation (message size limits)
✅ Added unit tests

**Next Step:** [Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)
