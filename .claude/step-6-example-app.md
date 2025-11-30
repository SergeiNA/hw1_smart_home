# Step 6: Create Example Application

**Prerequisites:**
- [Step 1: Protocol Design](step-1-protocol-design.md)
- [Step 2: Add TCP Support to Outlet](step-2-outlet-remote.md)
- [Step 3: Add UDP Support to Thermometer](step-3-thermometer-remote.md)
- [Step 4: Create Outlet Simulator](step-4-outlet-simulator.md)
- [Step 5: Create Thermometer Simulator](step-5-thermometer-simulator.md)

**Goal:** Create a complete example that demonstrates the entire system working together using the simulator API

---

## Overview

The example application will:
- Spawn device simulators programmatically
- Create a smart home with multiple rooms
- Use remote devices (outlets and thermometers)
- Control devices over the network
- Display device states and reports
- Handle errors gracefully
- Clean up automatically when finished

---

## 6.1 Create Complete Integration Example

**File:** `examples/remote_devices.rs`

```rust
use smart_home::create_home;
use smart_home::create_room;
use smart_home::simulators::{
    OutletSimulator, OutletSimulatorConfig,
    ThermometerSimulator, ThermometerSimulatorConfig,
    TemperaturePattern,
};
use smart_home::smart_devices::{Device, OutletDevice, TemperatureSensor, Celsius, Watt, OutletState};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::thermometr_remote::ThermometerRemote;
use smart_home::smart_home::SmartHome;
use smart_home::traits::Information;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║    Smart Home Remote Devices Example      ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Step 1: Spawn simulators
    let simulators = spawn_simulators();

    thread::sleep(Duration::from_millis(200));

    // Step 2: Create smart home with remote devices
    let home = create_smart_home(&simulators);

    println!("\n--- Initial Home State ---\n");
    print_home_report(&home);

    println!("\n\n--- Waiting for thermometer readings ---");
    thread::sleep(Duration::from_secs(2));

    println!("\n--- Updated Home State (with temperatures) ---\n");
    print_home_report(&home);

    println!("\n\n--- Controlling Devices ---\n");
    control_devices(&home);

    println!("\n\n--- Final Home State ---\n");
    print_home_report(&home);

    println!("\n✅ Example completed successfully!");
    println!("\nSimulators will be automatically stopped...");

    // Simulators automatically cleaned up when dropped
}

/// Spawn all device simulators and return their addresses
struct Simulators {
    living_room_outlet: OutletSimulator,
    living_room_outlet_addr: String,
    bedroom_outlet: OutletSimulator,
    bedroom_outlet_addr: String,
    living_room_thermo: ThermometerSimulator,
    bedroom_thermo: ThermometerSimulator,
}

fn spawn_simulators() -> Simulators {
    println!("🚀 Spawning device simulators...\n");

    // Spawn outlet simulators
    let living_room_outlet_config = OutletSimulatorConfig::new("127.0.0.1:0", 150);
    let living_room_outlet = OutletSimulator::spawn(living_room_outlet_config)
        .expect("Failed to spawn living room outlet simulator");
    let living_room_outlet_addr = living_room_outlet.addr().to_string();
    println!("   ✓ Living Room Outlet simulator: {}", living_room_outlet_addr);

    let bedroom_outlet_config = OutletSimulatorConfig::new("127.0.0.1:0", 300);
    let bedroom_outlet = OutletSimulator::spawn(bedroom_outlet_config)
        .expect("Failed to spawn bedroom outlet simulator");
    let bedroom_outlet_addr = bedroom_outlet.addr().to_string();
    println!("   ✓ Bedroom Outlet simulator: {}", bedroom_outlet_addr);

    // Spawn thermometer simulators
    let living_room_thermo_config = ThermometerSimulatorConfig::new(
        "127.0.0.1:19001",
        Duration::from_millis(500),
    )
    .with_pattern(TemperaturePattern::RandomWalk {
        min: 20.0,
        max: 24.0,
        step: 0.3,
    })
    .with_initial_temp(22.0);

    let living_room_thermo = ThermometerSimulator::spawn(living_room_thermo_config)
        .expect("Failed to spawn living room thermometer simulator");
    println!("   ✓ Living Room Thermometer simulator: 127.0.0.1:19001");

    let bedroom_thermo_config = ThermometerSimulatorConfig::new(
        "127.0.0.1:19002",
        Duration::from_millis(700),
    )
    .with_pattern(TemperaturePattern::RandomWalk {
        min: 18.0,
        max: 22.0,
        step: 0.2,
    })
    .with_initial_temp(20.0);

    let bedroom_thermo = ThermometerSimulator::spawn(bedroom_thermo_config)
        .expect("Failed to spawn bedroom thermometer simulator");
    println!("   ✓ Bedroom Thermometer simulator: 127.0.0.1:19002");

    Simulators {
        living_room_outlet,
        living_room_outlet_addr,
        bedroom_outlet,
        bedroom_outlet_addr,
        living_room_thermo,
        bedroom_thermo,
    }
}

fn create_smart_home(simulators: &Simulators) -> SmartHome {
    println!("\n🏠 Creating smart home with remote devices...");

    // Create remote devices
    let living_room_outlet = OutletRemote::new(
        "Living Room Outlet".to_string(),
        simulators.living_room_outlet_addr.clone(),
    ).expect("Failed to connect to living room outlet");

    let living_room_thermometer = ThermometerRemote::new(
        "Living Room Thermometer".to_string(),
        "127.0.0.1:19001".to_string(),
    ).expect("Failed to create living room thermometer");

    let bedroom_outlet = OutletRemote::new(
        "Bedroom Outlet".to_string(),
        simulators.bedroom_outlet_addr.clone(),
    ).expect("Failed to connect to bedroom outlet");

    let bedroom_thermometer = ThermometerRemote::new(
        "Bedroom Thermometer".to_string(),
        "127.0.0.1:19002".to_string(),
    ).expect("Failed to create bedroom thermometer");

    // Create home using macros
    let home = create_home!(
        "My Smart Home",
        {
            "Living Room",
            create_room!(
                "Living Room",
                "outlet" => Device::OutletTypeRemote(living_room_outlet),
                "thermometer" => Device::ThermometerTypeRemote(living_room_thermometer)
            )
        },
        {
            "Bedroom",
            create_room!(
                "Bedroom",
                "outlet" => Device::OutletTypeRemote(bedroom_outlet),
                "thermometer" => Device::ThermometerTypeRemote(bedroom_thermometer)
            )
        }
    );

    println!("   ✓ Created 2 rooms with 4 remote devices");

    home
}

fn print_home_report(home: &SmartHome) {
    println!("{}", home.info());
}

fn control_devices(home: &SmartHome) {
    println!("1. Turning on Living Room outlet...");
    if let Some(room) = home.get_room("Living Room") {
        if let Some(Device::OutletTypeRemote(outlet)) = room.get_device("outlet") {
            match outlet.turn_on() {
                Ok(_) => println!("   ✓ Living Room outlet turned ON"),
                Err(e) => println!("   ✗ Error: {}", e),
            }
        }
    }

    thread::sleep(Duration::from_millis(500));

    println!("\n2. Getting Living Room outlet state and power...");
    if let Some(room) = home.get_room("Living Room") {
        if let Some(Device::OutletTypeRemote(outlet)) = room.get_device("outlet") {
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

    thread::sleep(Duration::from_millis(500));

    println!("\n3. Turning on Bedroom outlet...");
    if let Some(room) = home.get_room("Bedroom") {
        if let Some(Device::OutletTypeRemote(outlet)) = room.get_device("outlet") {
            match outlet.turn_on() {
                Ok(_) => println!("   ✓ Bedroom outlet turned ON"),
                Err(e) => println!("   ✗ Error: {}", e),
            }
        }
    }

    thread::sleep(Duration::from_millis(500));

    println!("\n4. Reading all thermometers...");
    for room_name in &["Living Room", "Bedroom"] {
        if let Some(room) = home.get_room(room_name) {
            if let Some(Device::ThermometerTypeRemote(thermometer)) = room.view_device("thermometer") {
                let temp = thermometer.current_temperature();
                println!("   {}: {:.2}°C", room_name, temp);
            }
        }
    }

    thread::sleep(Duration::from_millis(500));

    println!("\n5. Switching Living Room outlet...");
    if let Some(room) = home.get_room("Living Room") {
        if let Some(Device::OutletTypeRemote(outlet)) = room.get_device("outlet") {
            match outlet.switch() {
                Ok(_) => println!("   ✓ Living Room outlet switched"),
                Err(e) => println!("   ✗ Error: {}", e),
            }
        }
    }
}
```

---

## 6.2 Create Simpler Example with Mixed Devices

**File:** `examples/mixed_devices.rs`

```rust
use smart_home::create_home;
use smart_home::create_room;
use smart_home::simulators::{OutletSimulator, OutletSimulatorConfig};
use smart_home::smart_devices::{Device, OutletDevice, OutletState, Watt, Celsius};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::traits::Information;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║      Mixed Local & Remote Devices         ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Spawn one remote outlet simulator
    let config = OutletSimulatorConfig::new("127.0.0.1:0", 200);
    let simulator = OutletSimulator::spawn(config)
        .expect("Failed to spawn simulator");

    println!("✓ Remote outlet simulator: {}\n", simulator.addr());

    thread::sleep(Duration::from_millis(100));

    // Connect to remote outlet
    let remote_outlet = OutletRemote::new(
        "Remote Outlet".to_string(),
        simulator.addr().to_string(),
    ).expect("Failed to connect");

    // Create home with both local and remote devices
    let mut home = create_home!(
        "My Home",
        {
            "Living Room",
            create_room!(
                "Living Room",
                "local_outlet" => Device::new_outlet(
                    "Local Outlet".to_string(),
                    OutletState::On,
                    150 as Watt
                ),
                "remote_outlet" => Device::OutletTypeRemote(remote_outlet),
                "thermometer" => Device::new_thermometer(
                    "Local Thermometer".to_string(),
                    22.5 as Celsius
                )
            )
        }
    );

    println!("📊 Home Report:\n{}\n", home.info());

    // Control remote outlet
    if let Some(room) = home.get_room("Living Room") {
        if let Some(Device::OutletTypeRemote(outlet)) = room.get_device("remote_outlet") {
            println!("Turning on remote outlet...");
            outlet.turn_on().expect("Failed to turn on");

            thread::sleep(Duration::from_millis(100));

            let state = outlet.state().expect("Failed to get state");
            let power = outlet.power_usage().expect("Failed to get power");

            println!("✓ Remote outlet: {:?}, {} watts\n", state, power);
        }
    }

    println!("📊 Updated Report:\n{}\n", home.info());

    println!("✅ Example completed!");
}
```

---

## 6.3 Create Error Handling Example

**File:** `examples/error_handling.rs`

```rust
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::OutletDevice;
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║        Error Handling Examples             ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Test 1: Connection refused
    println!("1. Attempting to connect to non-existent outlet...");
    let result = OutletRemote::new(
        "Broken Outlet".to_string(),
        "127.0.0.1:9999".to_string(),
    );

    match result {
        Ok(_) => println!("   ✗ Unexpected success!"),
        Err(e) => println!("   ✓ Expected error: {}\n", e),
    }

    // Test 2: Timeout handling
    // Note: Would need a simulator that doesn't respond to test this

    println!("✅ Error handling tests completed!");
}
```

---

## 6.4 Create Stress Test Example

**File:** `examples/stress_test.rs`

```rust
use smart_home::simulators::{
    OutletSimulator, OutletSimulatorConfig,
    ThermometerSimulator, ThermometerSimulatorConfig,
    TemperaturePattern,
};
use smart_home::smart_devices::outlet_remote::OutletRemote;
use smart_home::smart_devices::thermometr_remote::ThermometerRemote;
use smart_home::smart_devices::{OutletDevice, TemperatureSensor};
use std::thread;
use std::time::Duration;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║           Stress Test Example              ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Spawn multiple simulators
    println!("Spawning 5 outlet simulators and 3 thermometer simulators...\n");

    let mut outlet_sims = vec![];
    let mut thermo_sims = vec![];

    for i in 0..5 {
        let config = OutletSimulatorConfig::new("127.0.0.1:0", 100 + i * 50);
        let sim = OutletSimulator::spawn(config)
            .expect("Failed to spawn outlet");
        println!("   Outlet {}: {}", i + 1, sim.addr());
        outlet_sims.push(sim);
    }

    for i in 0..3 {
        let config = ThermometerSimulatorConfig::new(
            format!("127.0.0.1:{}", 19100 + i),
            Duration::from_millis(500),
        ).with_pattern(TemperaturePattern::RandomWalk {
            min: 18.0 + i as f32,
            max: 24.0 + i as f32,
            step: 0.3,
        });

        let sim = ThermometerSimulator::spawn(config)
            .expect("Failed to spawn thermometer");
        println!("   Thermometer {}: 127.0.0.1:{}", i + 1, 19100 + i);
        thermo_sims.push(sim);
    }

    println!("\n✓ All simulators spawned!\n");

    thread::sleep(Duration::from_millis(200));

    // Connect clients
    println!("Connecting clients...\n");
    let mut outlets = vec![];

    for (i, sim) in outlet_sims.iter().enumerate() {
        let outlet = OutletRemote::new(
            format!("Outlet {}", i + 1),
            sim.addr().to_string(),
        ).expect("Failed to connect");
        outlets.push(outlet);
    }

    let mut thermos = vec![];
    for i in 0..3 {
        let thermo = ThermometerRemote::new(
            format!("Thermo {}", i + 1),
            format!("127.0.0.1:{}", 19100 + i),
        ).expect("Failed to create thermometer");
        thermos.push(thermo);
    }

    // Perform operations
    println!("Performing rapid operations...\n");

    for _ in 0..10 {
        // Toggle all outlets
        for outlet in &mut outlets {
            let _ = outlet.switch();
        }

        // Read all thermometers
        for thermo in &thermos {
            let _ = thermo.current_temperature();
        }

        thread::sleep(Duration::from_millis(100));
    }

    println!("✓ Completed 10 rapid operation cycles\n");

    // Final status
    println!("Final status:");
    for (i, outlet) in outlets.iter().enumerate() {
        let state = outlet.state().unwrap_or_else(|_| OutletState::Off);
        let power = outlet.power_usage().unwrap_or(0);
        println!("   Outlet {}: {:?}, {} watts", i + 1, state, power);
    }

    for (i, thermo) in thermos.iter().enumerate() {
        let temp = thermo.current_temperature();
        println!("   Thermo {}: {:.2}°C", i + 1, temp);
    }

    println!("\n✅ Stress test completed!");
}
```

---

## 6.5 Run Examples

1. **Run the main integration example:**
   ```bash
   cargo run --example remote_devices
   ```

2. **Run the mixed devices example:**
   ```bash
   cargo run --example mixed_devices
   ```

3. **Run the error handling example:**
   ```bash
   cargo run --example error_handling
   ```

4. **Run the stress test:**
   ```bash
   cargo run --example stress_test
   ```

---

## 6.6 Expected Output

**remote_devices example:**

```
╔════════════════════════════════════════════╗
║    Smart Home Remote Devices Example      ║
╚════════════════════════════════════════════╝

🚀 Spawning device simulators...

   ✓ Living Room Outlet simulator: 127.0.0.1:51234
   ✓ Bedroom Outlet simulator: 127.0.0.1:51235
   ✓ Living Room Thermometer simulator: 127.0.0.1:19001
   ✓ Bedroom Thermometer simulator: 127.0.0.1:19002

🏠 Creating smart home with remote devices...
   ✓ Created 2 rooms with 4 remote devices

--- Initial Home State ---

Smart Home: My Smart Home
  Room: Living Room
    - Remote Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt
    - Thermometer: Living Room Thermometer - Current Temperature: 0.00°C
  Room: Bedroom
    - Remote Smart Outlet: Bedroom Outlet - Current State: Off, Power Usage: 0 Watt
    - Thermometer: Bedroom Thermometer - Current Temperature: 0.00°C


--- Waiting for thermometer readings ---

--- Updated Home State (with temperatures) ---

Smart Home: My Smart Home
  Room: Living Room
    - Remote Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt
    - Thermometer: Living Room Thermometer - Current Temperature: 22.15°C
  Room: Bedroom
    - Remote Smart Outlet: Bedroom Outlet - Current State: Off, Power Usage: 0 Watt
    - Thermometer: Bedroom Thermometer - Current Temperature: 20.08°C


--- Controlling Devices ---

1. Turning on Living Room outlet...
   ✓ Living Room outlet turned ON

2. Getting Living Room outlet state and power...
   State: On
   Power: 150 watts

3. Turning on Bedroom outlet...
   ✓ Bedroom outlet turned ON

4. Reading all thermometers...
   Living Room: 22.23°C
   Bedroom: 20.12°C

5. Switching Living Room outlet...
   ✓ Living Room outlet switched


--- Final Home State ---

Smart Home: My Smart Home
  Room: Living Room
    - Remote Smart Outlet: Living Room Outlet - Current State: Off, Power Usage: 0 Watt
    - Thermometer: Living Room Thermometer - Current Temperature: 22.31°C
  Room: Bedroom
    - Remote Smart Outlet: Bedroom Outlet - Current State: On, Power Usage: 300 Watt
    - Thermometer: Bedroom Thermometer - Current Temperature: 20.15°C

✅ Example completed successfully!

Simulators will be automatically stopped...
```

---

## Summary

✅ Created comprehensive integration examples
✅ Demonstrated programmatic simulator spawning
✅ Showed mixed local and remote device usage
✅ Added error handling examples
✅ Created stress test for multiple devices
✅ All cleanup happens automatically
✅ No need for separate terminal windows or scripts

**Advantages of this approach:**
- Everything in one process
- Automatic resource cleanup
- Easy to run and test
- No manual setup required
- Perfect for CI/CD
- Great developer experience

**Next Step:** [Step 7: Testing Strategy](step-7-testing.md)
