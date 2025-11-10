# Step 6: Create Example Application

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)
- [Step 3: Add UDP Support to Thermometer](step-3-thermometer-remote.md)
- [Step 4: Create Outlet Simulator](step-4-outlet-simulator.md)
- [Step 5: Create Thermometer Simulator](step-5-thermometer-simulator.md)

**Goal:** Create a complete example that demonstrates the entire system working together

---

## Overview

The example application will:
- Create a smart home with multiple rooms
- Use remote devices (outlets and thermometers)
- Control devices over the network
- Display device states and reports
- Handle errors gracefully

---

## 6.1 Create Example File

**File:** `examples/remote_devices.rs`

```rust
use smart_home::smart_devices::{Device, OutletDevice, TemperatureSensor};
use smart_home::smart_room::SmartRoom;
use smart_home::smart_home::SmartHome;
use smart_home::traits::Information;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║    Smart Home Remote Devices Example      ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("Prerequisites:");
    println!("  1. Start outlet simulators:");
    println!("     cargo run --bin outlet_simulator 127.0.0.1:8001 150");
    println!("     cargo run --bin outlet_simulator 127.0.0.1:8002 300");
    println!("  2. Start thermometer simulators:");
    println!("     cargo run --bin thermometer_simulator thermometer_living_room.toml");
    println!("     cargo run --bin thermometer_simulator thermometer_bedroom.toml");
    println!("\nPress Enter to continue...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Create smart home
    let home = create_smart_home();

    println!("\n--- Initial Home State ---\n");
    print_home_report(&home);

    println!("\n\n--- Waiting for initial thermometer readings ---");
    thread::sleep(Duration::from_secs(3));

    println!("\n--- Updated Home State (with temperatures) ---\n");
    print_home_report(&home);

    println!("\n\n--- Controlling Devices ---\n");
    control_devices(&home);

    println!("\n\n--- Final Home State ---\n");
    print_home_report(&home);

    println!("\n✅ Example completed successfully!");
}

fn create_smart_home() -> SmartHome {
    println!("\n🏠 Creating smart home with remote devices...");

    // Living Room devices
    let living_room_outlet = Device::new_outlet_remote(
        "Living Room Outlet".to_string(),
        "127.0.0.1:8001".to_string()
    );

    let living_room_thermometer = Device::new_thermometer_remote(
        "Living Room Thermometer".to_string(),
        "127.0.0.1:9001".to_string()
    ).expect("Failed to create living room thermometer");

    let mut living_room_devices = HashMap::new();
    living_room_devices.insert("outlet".to_string(), living_room_outlet);
    living_room_devices.insert("thermometer".to_string(), living_room_thermometer);

    let living_room = SmartRoom::new("Living Room".to_string(), living_room_devices);

    // Bedroom devices
    let bedroom_outlet = Device::new_outlet_remote(
        "Bedroom Outlet".to_string(),
        "127.0.0.1:8002".to_string()
    );

    let bedroom_thermometer = Device::new_thermometer_remote(
        "Bedroom Thermometer".to_string(),
        "127.0.0.1:9002".to_string()
    ).expect("Failed to create bedroom thermometer");

    let mut bedroom_devices = HashMap::new();
    bedroom_devices.insert("outlet".to_string(), bedroom_outlet);
    bedroom_devices.insert("thermometer".to_string(), bedroom_thermometer);

    let bedroom = SmartRoom::new("Bedroom".to_string(), bedroom_devices);

    // Create home
    let mut rooms = HashMap::new();
    rooms.insert("Living Room".to_string(), living_room);
    rooms.insert("Bedroom".to_string(), bedroom);

    println!("   ✓ Created 2 rooms with 4 remote devices");

    SmartHome::new("My Smart Home".to_string(), rooms)
}

fn print_home_report(home: &SmartHome) {
    println!("{}", home.info());
}

fn control_devices(home: &SmartHome) {
    // Get access to devices
    // Note: This requires adding a method to access rooms and devices
    // For now, we'll demonstrate with direct access if possible

    println!("1. Turning on Living Room outlet...");
    if let Some(room) = home.get_room("Living Room") {
        if let Some(device) = room.view_device("outlet") {
            if let Device::OutletType(outlet) = device {
                match outlet.turn_on() {
                    Ok(_) => println!("   ✓ Living Room outlet turned ON"),
                    Err(e) => println!("   ✗ Error: {}", e),
                }
            }
        }
    }

    thread::sleep(Duration::from_secs(1));

    println!("\n2. Getting Living Room outlet state...");
    if let Some(room) = home.get_room("Living Room") {
        if let Some(device) = room.view_device("outlet") {
            if let Device::OutletType(outlet) = device {
                match outlet.state() {
                    Ok(state) => println!("   State: {:?}", state),
                    Err(e) => println!("   ✗ Error: {}", e),
                }
                match outlet.power_usage() {
                    Ok(power) => println!("   Power: {} watts", power),
                    Err(e) => println!("   ✗ Error: {}", e),
                }
            }
        }
    }

    thread::sleep(Duration::from_secs(1));

    println!("\n3. Turning on Bedroom outlet...");
    if let Some(room) = home.get_room("Bedroom") {
        if let Some(device) = room.view_device("outlet") {
            if let Device::OutletType(outlet) = device {
                match outlet.turn_on() {
                    Ok(_) => println!("   ✓ Bedroom outlet turned ON"),
                    Err(e) => println!("   ✗ Error: {}", e),
                }
            }
        }
    }

    thread::sleep(Duration::from_secs(1));

    println!("\n4. Reading all thermometers...");
    for room_name in &["Living Room", "Bedroom"] {
        if let Some(room) = home.get_room(room_name) {
            if let Some(device) = room.view_device("thermometer") {
                if let Device::ThermometerType(thermometer) = device {
                    let temp = thermometer.current_temperature();
                    println!("   {}: {:.2}°C", room_name, temp);
                }
            }
        }
    }

    thread::sleep(Duration::from_secs(1));

    println!("\n5. Switching Living Room outlet...");
    if let Some(room) = home.get_room("Living Room") {
        if let Some(device) = room.view_device("outlet") {
            if let Device::OutletType(outlet) = device {
                match outlet.switch() {
                    Ok(_) => println!("   ✓ Living Room outlet switched"),
                    Err(e) => println!("   ✗ Error: {}", e),
                }
            }
        }
    }
}
```

---

## 6.2 Add Helper Methods to SmartHome

You may need to add these methods to `SmartHome`:

**File:** `src/smart_home.rs`

```rust
impl SmartHome {
    // If not already present, add:
    pub fn get_room(&self, room_name: &str) -> Option<&SmartRoom> {
        self.rooms.get(room_name)
    }

    pub fn get_room_mut(&mut self, room_name: &str) -> Option<&mut SmartRoom> {
        self.rooms.get_mut(room_name)
    }
}
```

**File:** `src/smart_room.rs`

```rust
impl SmartRoom {
    // Ensure these methods exist:
    pub fn view_device(&self, key: &str) -> Option<&Device> {
        self.devices.get(key)
    }

    pub fn get_device(&mut self, key: &str) -> Option<&mut Device> {
        self.devices.get_mut(key)
    }
}
```

---

## 6.3 Create Startup Script

Create a helper script to start all simulators:

**File:** `start_simulators.sh`

```bash
#!/bin/bash

# Start Smart Home Simulators

echo "Starting Smart Home simulators..."
echo ""

# Start outlet simulators
echo "Starting outlet simulators..."
cargo run --bin outlet_simulator 127.0.0.1:8001 150 &
PID_OUTLET1=$!
echo "  Living Room Outlet (PID: $PID_OUTLET1)"

cargo run --bin outlet_simulator 127.0.0.1:8002 300 &
PID_OUTLET2=$!
echo "  Bedroom Outlet (PID: $PID_OUTLET2)"

# Start thermometer simulators
echo ""
echo "Starting thermometer simulators..."
cargo run --bin thermometer_simulator thermometer_living_room.toml &
PID_THERMO1=$!
echo "  Living Room Thermometer (PID: $PID_THERMO1)"

cargo run --bin thermometer_simulator thermometer_bedroom.toml &
PID_THERMO2=$!
echo "  Bedroom Thermometer (PID: $PID_THERMO2)"

echo ""
echo "All simulators started!"
echo ""
echo "To stop all simulators, run:"
echo "  kill $PID_OUTLET1 $PID_OUTLET2 $PID_THERMO1 $PID_THERMO2"
echo ""
echo "Or save this to a file:"
echo "$PID_OUTLET1 $PID_OUTLET2 $PID_THERMO1 $PID_THERMO2" > .simulator_pids
echo "  Saved PIDs to .simulator_pids"
echo ""
echo "Now run:"
echo "  cargo run --example remote_devices"
```

Make it executable:
```bash
chmod +x start_simulators.sh
```

---

## 6.4 Create Stop Script

**File:** `stop_simulators.sh`

```bash
#!/bin/bash

if [ -f .simulator_pids ]; then
    echo "Stopping simulators..."
    PIDS=$(cat .simulator_pids)
    kill $PIDS 2>/dev/null
    rm .simulator_pids
    echo "Simulators stopped"
else
    echo "No .simulator_pids file found"
    echo "Kill manually or use: pkill -f outlet_simulator && pkill -f thermometer_simulator"
fi
```

Make it executable:
```bash
chmod +x stop_simulators.sh
```

---

## 6.5 Run the Complete System

### Step-by-step:

1. **Start all simulators:**
   ```bash
   ./start_simulators.sh
   ```

2. **Run the example:**
   ```bash
   cargo run --example remote_devices
   ```

3. **Stop all simulators:**
   ```bash
   ./stop_simulators.sh
   ```

### Or manually:

**Terminal 1:**
```bash
cargo run --bin outlet_simulator 127.0.0.1:8001 150
```

**Terminal 2:**
```bash
cargo run --bin outlet_simulator 127.0.0.1:8002 300
```

**Terminal 3:**
```bash
cargo run --bin thermometer_simulator thermometer_living_room.toml
```

**Terminal 4:**
```bash
cargo run --bin thermometer_simulator thermometer_bedroom.toml
```

**Terminal 5:**
```bash
cargo run --example remote_devices
```

---

## 6.6 Expected Output

**Example Application:**
```
╔════════════════════════════════════════════╗
║    Smart Home Remote Devices Example      ║
╚════════════════════════════════════════════╝

Prerequisites:
  1. Start outlet simulators:
     cargo run --bin outlet_simulator 127.0.0.1:8001 150
     cargo run --bin outlet_simulator 127.0.0.1:8002 300
  2. Start thermometer simulators:
     cargo run --bin thermometer_simulator thermometer_living_room.toml
     cargo run --bin thermometer_simulator thermometer_bedroom.toml

Press Enter to continue...

🏠 Creating smart home with remote devices...
   ✓ Created 2 rooms with 4 remote devices

--- Initial Home State ---

Smart Home: My Smart Home
  Room: Living Room
    - Living Room Outlet: Off, 0 watts
    - Living Room Thermometer: 20.00°C
  Room: Bedroom
    - Bedroom Outlet: Off, 0 watts
    - Bedroom Thermometer: 20.00°C


--- Waiting for initial thermometer readings ---

--- Updated Home State (with temperatures) ---

Smart Home: My Smart Home
  Room: Living Room
    - Living Room Outlet: Off, 0 watts
    - Living Room Thermometer: 22.34°C
  Room: Bedroom
    - Bedroom Outlet: Off, 0 watts
    - Bedroom Thermometer: 19.87°C


--- Controlling Devices ---

1. Turning on Living Room outlet...
   ✓ Living Room outlet turned ON

2. Getting Living Room outlet state...
   State: On
   Power: 150 watts

3. Turning on Bedroom outlet...
   ✓ Bedroom outlet turned ON

4. Reading all thermometers...
   Living Room: 22.56°C
   Bedroom: 19.92°C

5. Switching Living Room outlet...
   ✓ Living Room outlet switched


--- Final Home State ---

Smart Home: My Smart Home
  Room: Living Room
    - Living Room Outlet: Off, 0 watts
    - Living Room Thermometer: 22.78°C
  Room: Bedroom
    - Bedroom Outlet: On, 300 watts
    - Bedroom Thermometer: 20.01°C

✅ Example completed successfully!
```

---

## 6.7 Add Error Handling Example

Create an example that demonstrates error handling:

**File:** `examples/error_handling.rs`

```rust
use smart_home::smart_devices::Device;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Testing error handling...\n");

    // Try to connect to non-existent simulator
    println!("1. Attempting to connect to non-existent outlet...");
    let mut outlet = Device::new_outlet_remote(
        "Broken Outlet".to_string(),
        "127.0.0.1:9999".to_string()
    );

    if let Device::OutletType(ref mut o) = outlet {
        match o.turn_on() {
            Ok(_) => println!("   ✓ Success (unexpected!)"),
            Err(e) => println!("   ✗ Error (expected): {}", e),
        }
    }

    println!("\n2. Attempting to read from outlet with timeout...");
    if let Device::OutletType(ref o) = outlet {
        match o.state() {
            Ok(state) => println!("   State: {:?}", state),
            Err(e) => println!("   ✗ Error: {}", e),
        }
    }

    println!("\n✅ Error handling test completed");
}
```

---

## Summary

✅ Created comprehensive example application
✅ Demonstrated full system integration
✅ Created helper scripts for managing simulators
✅ Showed device control over network
✅ Added error handling examples
✅ Documented expected output

**Next Step:** [Step 7: Testing Strategy](step-7-testing.md)
