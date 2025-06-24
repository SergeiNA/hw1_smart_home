# hw1_smart_home

## Homework Assignment

### Smart Home Library Starter Project

### 🎯 Goal

Create a starter implementation of a “Smart Home” library, along with a simple example demonstrating its functionality.

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

###  📦 Project Structure

The library and its usage example are implemented within a single Cargo package:
	•	The library is implemented as a lib crate.
	•	The example is implemented as a bin crate.

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

### 🧩 Library Requirements

1. Smart Thermometer

Implement a type representing a smart thermometer. It should provide:
	•	A constructor that accepts initial field values.
	•	A method to return the current temperature (use an arbitrary number).

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

2. Smart Outlet

Implement a type representing a smart power outlet. It should provide:
	•	A constructor that accepts initial field values.
	•	Methods to turn the outlet on/off.
	•	A method to query current state (on or off).
	•	A method to return the current power usage:
	•	If off → returns 0.
	•	If on → returns an arbitrary number.

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

3. Smart Device

Implement an enum or struct that contains either a smart thermometer or a smart outlet, and provides:
	•	A method to print the state of the device to standard output.

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

4. Smart Room

Implement a type representing a room that contains an array of smart devices. It should provide:
	•	A constructor that accepts an array of devices.
	•	Methods to:
	•	Get a reference to a device by index.
	•	Get a mutable reference to a device by index.
	•	Print a status report of all devices in the room.

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

5. Smart Home

Implement a type representing a smart home, which contains an array of rooms. It should provide:
	•	A constructor that accepts an array of rooms.
	•	Methods to:
	•	Get a reference to a room by index.
	•	Get a mutable reference to a room by index.
	•	Print a status report of all rooms.

You can choose arbitrary array sizes.
If an index is out of bounds, the application must terminate with a panic!().

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

### 🔧 Example Binary Requirements

Implemented as a bin crate.
	•	Create an instance of the smart home and print a full report of its contents.
	•	Modify the home: turn off a smart outlet in one of the rooms.
	•	Print the report again.

⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻⸻

### ✅ Evaluation Criteria
	•	The package builds successfully with cargo build.
	•	The binary example runs successfully and prints the smart home report.
	•	Commands cargo clippy and cargo fmt --check return with no warnings or errors.
	•	Unit tests are present and pass successfully.
