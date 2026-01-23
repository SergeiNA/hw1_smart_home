# hw5_smart_home_web

## Homework Assignment

### Smart Home Web Service

### Goal

Turn the smart home into a web service.

---

## Description

Implement a backend service for managing the smart home and a frontend application for interacting with it.

### Backend Service

The interaction technology with the backend service (gRPC, REST, GraphQL, ...) is chosen arbitrarily.

The backend API provides access to all basic functionality of the smart home library:

* Add/remove/list rooms in the home and get information about a specific room.
* Add/remove/list devices in a room and get information about a specific device.
* Get a report about the home.

Functional tests must be present that communicate with the backend and verify its responses.

### Frontend Application

* Displays the list of rooms in the home.
* Allows navigation to a specific room or adding a new room.
* Displays the list of devices in a room.
* Allows navigation to a specific device or adding a new device.
* Allows requesting a report about the home state.

---

## Evaluation Criteria

* Workspace builds successfully.
* Tests pass successfully.
* `cargo clippy` and `cargo fmt --check` return without errors or warnings.

