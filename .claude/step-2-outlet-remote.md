# Step 2: Add TCP Support to Smart Outlet

**Prerequisites:** [Step 1: Protocol Design](step-1-protocol-design.md)

**Goal:** Extend the `Outlet` struct to support both local and remote (TCP) operation modes

---

## Overview

In this step, we'll modify the existing `Outlet` implementation to support two modes:
- **Local mode**: Current in-memory implementation (no changes to existing behavior)
- **Remote mode**: Communicates with a TCP server using binary protocol

This ensures backward compatibility while adding network capabilities.

---

## 2.1 Define Operation Mode

**File:** `src/smart_devices/outlet.rs`

Add the mode enum and modify the struct:

```rust
use crate::protocol::outlet::{OutletRequest, OutletResponse};
use std::net::TcpStream;
use std::time::Duration;

/// Operation mode for the outlet
#[derive(Debug)]
pub enum OutletMode {
    /// Direct in-memory access (current behavior)
    Local,
    /// TCP communication with remote device
    Remote { tcp_addr: String },
}

// Update the Outlet struct to include mode and optional TCP connection
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

**Note:** The `Outlet` struct can no longer derive `Clone` due to `TcpStream`. You'll need to remove `Clone` from the derive macro or implement `Clone` manually (creating a new connection).

---

## 2.2 Update Constructors

Keep the existing constructor for local mode and add a new one for remote:

```rust
impl Outlet {
    /// Create a new outlet in local mode (existing behavior)
    pub fn new(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        Outlet {
            name,
            mode: OutletMode::Local,
            state: initial_state,
            power_usage,
            tcp_connection: None,
        }
    }

    /// Create a new outlet in remote mode (TCP communication)
    pub fn new_remote(name: String, tcp_addr: String) -> Self {
        Outlet {
            name,
            mode: OutletMode::Remote { tcp_addr },
            state: OutletState::Off,  // Default, will be fetched from remote
            power_usage: 0,           // Default, will be fetched from remote
            tcp_connection: None,     // Lazy connection on first use
        }
    }
}
```

---

## 2.3 Add Connection Management

Implement helper methods for TCP connection handling:

```rust
impl Outlet {
    /// Get existing connection or establish a new one
    fn get_or_connect(&mut self) -> Result<&mut TcpStream, OutletError> {
        if self.tcp_connection.is_none() {
            if let OutletMode::Remote { tcp_addr } = &self.mode {
                let mut stream = TcpStream::connect(tcp_addr)
                    .map_err(OutletError::NetworkError)?;

                // Set timeouts to prevent hanging
                stream.set_read_timeout(Some(Duration::from_secs(5)))
                    .map_err(OutletError::NetworkError)?;
                stream.set_write_timeout(Some(Duration::from_secs(5)))
                    .map_err(OutletError::NetworkError)?;

                self.tcp_connection = Some(stream);
            }
        }

        self.tcp_connection.as_mut()
            .ok_or_else(|| OutletError::NetworkError(
                std::io::Error::new(std::io::ErrorKind::NotConnected, "Not connected")
            ))
    }

    /// Send a request and receive a response
    fn send_request(&mut self, request: OutletRequest) -> Result<OutletResponse, OutletError> {
        let stream = self.get_or_connect()?;

        // Send request
        request.send(stream).map_err(OutletError::NetworkError)?;

        // Receive response
        let response = OutletResponse::receive(stream)
            .map_err(OutletError::NetworkError)?;

        Ok(response)
    }

    /// Reset connection (for retry logic)
    fn reset_connection(&mut self) {
        self.tcp_connection = None;
    }
}
```

---

## 2.4 Define Error Type

Add a custom error type for outlet operations:

```rust
#[derive(Debug)]
pub enum OutletError {
    NetworkError(std::io::Error),
    ProtocolError(String),
}

impl std::fmt::Display for OutletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutletError::NetworkError(e) => write!(f, "Network error: {}", e),
            OutletError::ProtocolError(e) => write!(f, "Protocol error: {}", e),
        }
    }
}

impl std::error::Error for OutletError {}
```

---

## 2.5 Update Trait Implementation

Modify the `OutletDevice` trait methods to support both modes:

**Important:** This will require changing the trait signature to return `Result`. You can either:
- Option A: Keep local mode infallible, only remote returns `Result`
- Option B: Make all methods return `Result` for consistency

Here's Option B (recommended):

```rust
// Update trait definition in src/smart_devices/outlet.rs
pub trait OutletDevice: Information {
    fn turn_on(&mut self) -> Result<(), OutletError>;
    fn turn_off(&mut self) -> Result<(), OutletError>;
    fn switch(&mut self) -> Result<(), OutletError>;
    fn state(&self) -> Result<OutletState, OutletError>;
    fn power_usage(&self) -> Result<Watt, OutletError>;
}

impl OutletDevice for Outlet {
    fn state(&self) -> Result<OutletState, OutletError> {
        match &self.mode {
            OutletMode::Local => Ok(self.state),
            OutletMode::Remote { .. } => {
                let mut outlet_mut = self;
                let response = outlet_mut.send_request(OutletRequest::GetState)?;
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

    fn turn_off(&mut self) -> Result<(), OutletError> {
        match &self.mode {
            OutletMode::Local => {
                self.state = OutletState::Off;
                Ok(())
            }
            OutletMode::Remote { .. } => {
                let response = self.send_request(OutletRequest::TurnOff)?;
                match response {
                    OutletResponse::Ok => Ok(()),
                    OutletResponse::Error(e) => Err(OutletError::ProtocolError(e)),
                    _ => Err(OutletError::ProtocolError("Unexpected response".to_string())),
                }
            }
        }
    }

    fn switch(&mut self) -> Result<(), OutletError> {
        match &self.mode {
            OutletMode::Local => {
                self.state = match self.state {
                    OutletState::On => OutletState::Off,
                    OutletState::Off => OutletState::On,
                };
                Ok(())
            }
            OutletMode::Remote { .. } => {
                let response = self.send_request(OutletRequest::Switch)?;
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
                Ok(if self.state == OutletState::On {
                    self.power_usage
                } else {
                    0
                })
            }
            OutletMode::Remote { .. } => {
                let mut outlet_mut = self;
                let response = outlet_mut.send_request(OutletRequest::GetPower)?;
                match response {
                    OutletResponse::Power(watts) => Ok(watts),
                    OutletResponse::Error(e) => Err(OutletError::ProtocolError(e)),
                    _ => Err(OutletError::ProtocolError("Unexpected response".to_string())),
                }
            }
        }
    }
}
```

**Note:** There's a borrow checker issue with `state()` and `power_usage()` taking `&self`. You may need to:
- Make them take `&mut self`
- Use `RefCell` for interior mutability
- Cache the TCP connection differently

---

## 2.6 Update Device Enum Constructor

**File:** `src/smart_devices.rs`

Add a constructor for remote outlets:

```rust
impl Device {
    pub fn new_outlet(name: String, initial_state: OutletState, power_usage: Watt) -> Self {
        Device::OutletType(Outlet::new(name, initial_state, power_usage))
    }

    // Add this new constructor
    pub fn new_outlet_remote(name: String, tcp_addr: String) -> Self {
        Device::OutletType(Outlet::new_remote(name, tcp_addr))
    }

    pub fn new_thermometer(name: String, initial_temperature: Celsius) -> Self {
        Device::ThermometerType(Thermometer::new(name, initial_temperature))
    }
}
```

---

## 2.7 Fix Existing Tests

Update existing tests to use `.unwrap()` on Result returns:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlet_turn_on_off_test() {
        let mut outlet = Outlet::new("Living Room".to_string(), OutletState::Off, 100);

        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);

        outlet.turn_on().unwrap();
        assert_eq!(outlet.power_usage().unwrap(), 100);
        assert_eq!(outlet.state().unwrap(), OutletState::On);

        outlet.turn_off().unwrap();
        assert_eq!(outlet.power_usage().unwrap(), 0);
        assert_eq!(outlet.state().unwrap(), OutletState::Off);
    }
}
```

---

## Verification

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Run existing tests:**
   ```bash
   cargo test outlet
   ```

3. **Check that local mode still works:**
   ```bash
   cargo run --example basic_usage
   ```

---

## Common Issues & Solutions

### Issue 1: Borrow Checker Errors with `&self` methods

**Problem:** Can't get `&mut self` in methods that take `&self`

**Solutions:**
- Use `Cell<Option<TcpStream>>` or `RefCell` for interior mutability
- Change trait to use `&mut self` for all methods
- Cache connection at a higher level

### Issue 2: Clone trait removed

**Problem:** `Device` enum can no longer be cloned

**Solutions:**
- Remove `Clone` from Device enum
- Implement `Clone` manually to create new connections
- Use `Arc<Mutex<Device>>` for sharing

---

## Summary

✅ Added `OutletMode` enum for local/remote switching
✅ Created `Outlet::new_remote()` constructor
✅ Implemented TCP connection management with lazy connection
✅ Added `OutletError` type for error handling
✅ Updated `OutletDevice` trait to return `Result`
✅ Maintained backward compatibility with local mode

**Next Step:** [Step 3: Add UDP Support to Thermometer](step-3-thermometer-remote.md)
