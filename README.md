# hw3_smart_home_devices

## Homework Assignment

### Smart Home: Remote Device Interaction & Simulators

### 🌟 Goal

Implement remote interaction logic for smart devices (outlet & thermometer) and create device simulators for testing.

---

### 📦 Project Structure

All code is implemented in a single Cargo package:

- The library is implemented as a `lib` crate.
- The simulators and example application are implemented as `bin` crates.

---

### 🮩 Library Requirements

#### 1. 🔌 Smart Outlet

- Functionality remains the same:
	- Turn on/off.
	- Query power consumption.
- Interaction is synchronous, via TCP.
- The outlet can use:
	- Real TCP communication, or
	- Simulation mode for testing.

---

#### 2. 🌡️ Smart Thermometer

- Functionality remains the same:
	- Return current temperature.
- Temperature values are received over UDP in a parallel thread.
- The thread is:
	- Started when the thermometer is created.
	- Stopped when the thermometer object is destroyed.
- The thermometer always returns the latest received value.
- The thermometer can also simulate remote data reception (for testing).

---

### 🖥️ Simulator Requirements

#### 1. Smart Outlet Simulator

- Reads the TCP listening address from command-line arguments.
- Implements non-blocking TCP communication.
- Maintains outlet state (on/off).
- Supports multiple concurrent client connections.

#### 2. Smart Thermometer Simulator

- Implements non-blocking UDP communication.
- Reads the target UDP address and sending period from a configuration file.
- Sends random temperature values to the specified address at the given interval.

---

### 🔧 Example Binary Requirements

Add an example smart home application that uses both outlets and thermometers connected to simulators.

- The example must:
	- Print a report of the smart home state if simulators are running.
	- Report an error if any device fails to retrieve data.

---

### ✅ Evaluation Criteria

- The package builds successfully with `cargo build`.
- The example application runs and prints smart home reports.
- `cargo clippy` and `cargo fmt --check` return without warnings or errors.
- Unit tests are implemented and pass successfully.