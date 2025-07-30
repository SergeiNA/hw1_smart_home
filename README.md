# hw2\_smart\_home\_plus

## Homework Assignment

### Smart Home: Dynamic Extensions & Error Handling

### 🌟 Goal

Extend the functionality of the Smart Home library using features from the Rust standard library:

* Add proper error handling.
* Switch to dynamic, string-based collections.
* Enable runtime changes to the smart home structure.

---

### 📦 Project Structure

All code is implemented in a single Cargo package:

* The library is implemented as a `lib` crate.
* The example application is implemented as a `bin` crate.

---

### 🮩 Library Requirements

#### 1. ✅ Error Handling

Update methods that retrieve rooms or devices:

* Replace panics with safe returns using `Option` or `Result`.
* Implement a custom error type for `SmartHome` lookup operations.
* The error type must implement `std::error::Error`.

---

#### 2. 🔑 Key-Based Storage

Replace fixed-size arrays with dynamic, key-based collections:

* Use associative containers (e.g., `HashMap<String, ...>`) from `std::collections`.
* Use strings as keys for both rooms and devices.

---

#### 3. 🔄 Dynamic Modifications

Support modifying the smart home at runtime:

* Add methods to insert/remove devices in a room.
* Add methods to insert/remove rooms in the home.
* Add a method on the smart home to retrieve a reference to a device by (room\_name, device\_name).

	* Return an appropriate error if lookup fails.

---

#### 4. 🧠 Trait Implementations

* Implement the `Debug` trait for all types.
* Implement the `From` trait for converting smart outlet and smart thermometer into a smart device enum.

---

#### 5. 🚰 Macro for Room Creation

Write a macro to simplify room construction:

* Accept key-value pairs like `("outlet1", SmartOutlet::new(...))`
* Return a `SmartRoom` with the devices mapped by the given keys.

---

#### 6. 📊 Status Report Abstraction

* Extract status reporting into a trait.
* Implement the trait for all types that can generate a report: device, room, and home.

---

### 🔧 Example Binary Requirements

Implemented as a `bin` crate.

Demonstrate:

* Adding and removing rooms at runtime.
* Adding and removing devices at runtime.
* Retrieving and printing a report for:

	* The entire smart home.
	* A single room.
	* A single device.

Add a helper function that takes any object implementing the report trait and prints its report.

Also demonstrate error handling when looking up rooms or devices.

---

### ✅ Evaluation Criteria

* The package builds successfully with `cargo build`.
* The example application runs and prints smart home reports.
* `cargo clippy` and `cargo fmt --check` return without warnings or errors.
* Unit tests are implemented and pass successfully.
