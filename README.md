# hw4_smart_home_plus

## Homework Assignment

### Smart Home: Dynamic Extensions, Error Handling & Design Patterns

### Goal

Extend the functionality of the Smart Home library using features from the Rust standard library and design patterns:

* Add proper error handling.
* Switch to dynamic, string-based collections.
* Enable runtime changes to the smart home structure.
* Apply common design patterns: **Builder**, **Composite**, and **Observer**.

---

## Project Structure

All code is implemented in a single Cargo package:

* The library is implemented as a `lib` crate.
* The example application is implemented as a `bin` crate.

---

## Library Requirements

### 1. Error Handling

Update methods that retrieve rooms or devices:

* Replace panics with safe returns using `Option` or `Result`.
* Implement a custom error type for `SmartHome` lookup operations.
* The error type must implement `std::error::Error`.

### 2. Key-Based Storage

Replace fixed-size arrays with dynamic, key-based collections:

* Use associative containers (e.g., `HashMap<String, ...>`) from `std::collections`.
* Use strings as keys for both rooms and devices.

### 3. Dynamic Modifications

Support modifying the smart home at runtime:

* Add methods to insert/remove devices in a room.
* Add methods to insert/remove rooms in the home.
* Add a method on the smart home to retrieve a reference to a device by `(room_name, device_name)`.
	* Return an appropriate error if lookup fails.

### 4. Trait Implementations

* Implement the `Debug` trait for all types.
* Implement the `From` trait for converting smart outlet and smart thermometer into a smart device enum.

### 5. Macro for Room Creation

Write a macro to simplify room construction:

* Accept key-value pairs like `( "outlet1", SmartOutlet::new(...) )`
* Return a `SmartRoom` with the devices mapped by the given keys.

### 6. Status Report Abstraction

* Extract status reporting into a trait.
* Implement the trait for all types that can generate a report: device, room, and home.

---

## Additional Design Pattern Requirements

### 1. Builder Pattern

* Implement a builder for the smart home.
* The builder must enforce compile-time restrictions: before the first room is added, devices cannot be added.
* Use a typestate pattern (e.g., `SmartHomeBuilder<NoRooms>` → `SmartHomeBuilder<HasRoom>`).

### 2. Composite Pattern

* Implement a composite structure for building reports.
* Use static polymorphism (generics).
* `report()` should print structured reports of all added objects.

### 3. Observer Pattern

* Add callbacks to rooms that trigger when new devices are added.
* Use dynamic polymorphism (trait objects).
* Subscribers can be either concrete objects or closures.

---

## Example Binary Requirements

* Demonstrate adding and removing rooms at runtime.
* Demonstrate adding and removing devices at runtime.
* Show how to retrieve and print a report for:
	* The entire smart home.
	* A single room.
	* A single device.
* Add a helper function that takes any object implementing the report trait and prints its report.
* Demonstrate error handling when looking up rooms or devices.
* Demonstrate builder, composite, and observer patterns in action.

---

## Evaluation Criteria

* The package builds successfully with `cargo build`.
* The example application runs and prints smart home reports.
* `cargo clippy` and `cargo fmt --check` return without warnings or errors.
* Unit tests are implemented and pass successfully.

---

## Implementation Hints

* Prefer `HashMap<String, Box<dyn DeviceTrait>>` or an enum-wrapped device type for heterogeneous device collections.
* For the builder, enforce restrictions using typestate with generic parameters.
* For the observer, provide APIs for both closures and subscriber objects.
* For the composite report, define a `Reporter` trait and implement it for `Device`, `Room`, and `SmartHome`.
* Ensure the custom error type implements `Display` and `Error`.

---
