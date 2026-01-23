# Frontend Implementation Plan (Yew)

A minimalistic Yew frontend for the Smart Home web service.

---

## Requirements Checklist

- [ ] Display list of rooms
- [ ] Navigate to specific room
- [ ] Add new room
- [ ] Display list of devices in room
- [ ] Navigate to specific device
- [ ] Add new device
- [ ] Request home report

---

## Tech Stack

| Tool | Purpose |
|------|---------|
| Yew | Rust frontend framework |
| Trunk | Build tool / dev server |
| gloo-net | HTTP requests |
| wasm-bindgen | WASM bindings |

---

## Current Workspace Structure

```
hw1_smart_home/
├── Cargo.toml              # Workspace root
├── smart_home/             # Library crate
│   ├── Cargo.toml
│   └── src/
└── server/                 # Backend crate
    ├── Cargo.toml
    └── src/
```

## Target Structure

```
hw1_smart_home/
├── Cargo.toml              # Workspace root (UPDATED)
├── smart_home/             # Library crate
│   ├── Cargo.toml
│   └── src/
├── server/                 # Backend crate
│   ├── Cargo.toml
│   └── src/
└── web/                    # Frontend crate (NEW)
    ├── Cargo.toml
    ├── Trunk.toml
    ├── index.html
    ├── style.css
    └── src/
```

---

## Step-by-Step Setup

### Step 1: Update workspace Cargo.toml

Edit `hw1_smart_home/Cargo.toml`:

```toml
[workspace]
members = ["smart_home", "server", "web"]
resolver = "2"
```

### Step 2: Create frontend directory structure

```bash
# From hw1_smart_home/
mkdir -p web/src/components
```

### Step 3: Create frontend Cargo.toml

Create `web/Cargo.toml`:

```toml
[package]
name = "web"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
gloo-net = "0.5"
wasm-bindgen-futures = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
web-sys = "0.3"
```

### Step 4: Create Trunk.toml

`web/Trunk.toml`:

```toml
[build]
target = "index.html"
dist = "dist"

[serve]
address = "127.0.0.1"
port = 8080

[[proxy]]
rewrite = "/api/"
backend = "http://127.0.0.1:3000/api/"
```

### Step 5: Create index.html

`web/index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>Smart Home</title>
    <link data-trunk rel="css" href="style.css" />
</head>
<body>
</body>
</html>
```

### Step 6: Create minimal CSS

`web/style.css`:

```css
body {
    font-family: system-ui, sans-serif;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
}

button {
    padding: 8px 16px;
    margin: 4px;
    cursor: pointer;
}

input {
    padding: 8px;
    margin: 4px;
}

.room-list, .device-list {
    list-style: none;
    padding: 0;
}

.room-item, .device-item {
    padding: 10px;
    margin: 5px 0;
    background: #f5f5f5;
    border-radius: 4px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.back-btn {
    margin-bottom: 20px;
}

.report {
    background: #e8f5e9;
    padding: 15px;
    border-radius: 4px;
    white-space: pre-wrap;
    font-family: monospace;
}

.error {
    color: red;
    padding: 10px;
}
```

---

## Quick Setup (Copy-Paste Commands)

Run all commands from the `hw1_smart_home/` directory:

```bash
# Step 1: Create directory structure
mkdir -p web/src/components

# Step 2: Update workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = ["smart_home", "server", "web"]
resolver = "2"
EOF

# Step 3: Create web/Cargo.toml
cat > web/Cargo.toml << 'EOF'
[package]
name = "web"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
gloo-net = "0.5"
wasm-bindgen-futures = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
web-sys = "0.3"
EOF

# Step 4: Create web/Trunk.toml
cat > web/Trunk.toml << 'EOF'
[build]
target = "index.html"
dist = "dist"

[serve]
address = "127.0.0.1"
port = 8080

[[proxy]]
rewrite = "/api/"
backend = "http://127.0.0.1:3000/api/"
EOF

# Step 5: Create web/index.html
cat > web/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>Smart Home</title>
    <link data-trunk rel="css" href="style.css" />
</head>
<body>
</body>
</html>
EOF

# Step 6: Create placeholder main.rs
cat > web/src/main.rs << 'EOF'
fn main() {
    println!("Smart Home Web - TODO: implement");
}
EOF

# Step 7: Install trunk (if not installed)
cargo install trunk

# Step 8: Add wasm target
rustup target add wasm32-unknown-unknown

# Step 9: Verify workspace builds
cargo build
```

---

## Project Structure

```
web/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── style.css
└── src/
    ├── main.rs          # Entry point
    ├── api.rs           # API client
    ├── models.rs        # Data types
    └── components/
        ├── mod.rs
        ├── home.rs      # Home view (room list)
        ├── room.rs      # Room view (device list)
        ├── device.rs    # Device view
        └── report.rs    # Report view
```

---

## Implementation

### 1. Models (`src/models.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RoomResponse {
    pub name: String,
    pub device_count: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct RoomsListResponse {
    pub rooms: Vec<RoomResponse>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DeviceResponse {
    pub name: String,
    pub device_type: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DevicesListResponse {
    pub room_name: String,
    pub devices: Vec<DeviceResponse>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct ReportResponse {
    pub report: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateRoomRequest {
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub device_type: DeviceTypeRequest,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "params")]
pub enum DeviceTypeRequest {
    #[serde(rename = "outlet")]
    Outlet { description: String },
    #[serde(rename = "thermometer")]
    Thermometer { temperature: f64 },
}
```

### 2. API Client (`src/api.rs`)

```rust
use gloo_net::http::Request;
use crate::models::*;

const BASE_URL: &str = "/api/v1";

pub async fn fetch_rooms() -> Result<Vec<RoomResponse>, String> {
    let response = Request::get(&format!("{}/rooms", BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: RoomsListResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(data.rooms)
}

pub async fn create_room(name: &str) -> Result<(), String> {
    Request::post(&format!("{}/rooms", BASE_URL))
        .json(&CreateRoomRequest { name: name.to_string() })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn delete_room(name: &str) -> Result<(), String> {
    Request::delete(&format!("{}/rooms/{}", BASE_URL, name))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn fetch_devices(room: &str) -> Result<Vec<DeviceResponse>, String> {
    let response = Request::get(&format!("{}/rooms/{}/devices", BASE_URL, room))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: DevicesListResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(data.devices)
}

pub async fn create_device(room: &str, name: &str, device_type: DeviceTypeRequest) -> Result<(), String> {
    Request::post(&format!("{}/rooms/{}/devices", BASE_URL, room))
        .json(&CreateDeviceRequest { name: name.to_string(), device_type })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn delete_device(room: &str, device: &str) -> Result<(), String> {
    Request::delete(&format!("{}/rooms/{}/devices/{}", BASE_URL, room, device))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn fetch_report() -> Result<String, String> {
    let response = Request::get(&format!("{}/report", BASE_URL))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: ReportResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(data.report)
}
```

### 3. Components Module (`src/components/mod.rs`)

```rust
pub mod home;
pub mod room;
pub mod device;
pub mod report;

pub use home::Home;
pub use room::Room;
pub use device::Device;
pub use report::Report;
```

### 4. Home Component (`src/components/home.rs`)

```rust
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::models::RoomResponse;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_select_room: Callback<String>,
    pub on_show_report: Callback<()>,
}

#[function_component(Home)]
pub fn home(props: &Props) -> Html {
    let rooms = use_state(Vec::new);
    let error = use_state(|| None::<String>);
    let new_room = use_state(String::new);

    // Fetch rooms on mount
    {
        let rooms = rooms.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::fetch_rooms().await {
                    Ok(data) => rooms.set(data),
                    Err(e) => error.set(Some(e)),
                }
            });
        });
    }

    let on_add_room = {
        let new_room = new_room.clone();
        let rooms = rooms.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let name = (*new_room).clone();
            if name.is_empty() { return; }
            let new_room = new_room.clone();
            let rooms = rooms.clone();
            let error = error.clone();
            spawn_local(async move {
                match api::create_room(&name).await {
                    Ok(_) => {
                        new_room.set(String::new());
                        if let Ok(data) = api::fetch_rooms().await {
                            rooms.set(data);
                        }
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_delete_room = {
        let rooms = rooms.clone();
        let error = error.clone();
        Callback::from(move |name: String| {
            let rooms = rooms.clone();
            let error = error.clone();
            spawn_local(async move {
                match api::delete_room(&name).await {
                    Ok(_) => {
                        if let Ok(data) = api::fetch_rooms().await {
                            rooms.set(data);
                        }
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_input = {
        let new_room = new_room.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            new_room.set(input.value());
        })
    };

    html! {
        <div>
            <h1>{"Smart Home"}</h1>

            <button onclick={props.on_show_report.reform(|_| ())}>
                {"View Report"}
            </button>

            <h2>{"Rooms"}</h2>

            if let Some(err) = (*error).clone() {
                <div class="error">{err}</div>
            }

            <div>
                <input
                    type="text"
                    placeholder="New room name"
                    value={(*new_room).clone()}
                    oninput={on_input}
                />
                <button onclick={on_add_room}>{"Add Room"}</button>
            </div>

            <ul class="room-list">
                { for (*rooms).iter().map(|room| {
                    let name = room.name.clone();
                    let on_select = props.on_select_room.reform(move |_| name.clone());
                    let name_del = room.name.clone();
                    let on_delete = on_delete_room.reform(move |_| name_del.clone());
                    html! {
                        <li class="room-item">
                            <span onclick={on_select} style="cursor: pointer;">
                                {&room.name} {" ("}{room.device_count}{" devices)"}
                            </span>
                            <button onclick={on_delete}>{"Delete"}</button>
                        </li>
                    }
                })}
            </ul>
        </div>
    }
}
```

### 5. Room Component (`src/components/room.rs`)

```rust
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::models::{DeviceResponse, DeviceTypeRequest};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub room_name: String,
    pub on_back: Callback<()>,
    pub on_select_device: Callback<String>,
}

#[function_component(Room)]
pub fn room(props: &Props) -> Html {
    let devices = use_state(Vec::new);
    let error = use_state(|| None::<String>);
    let new_device = use_state(String::new);
    let device_type = use_state(|| "outlet".to_string());

    let room_name = props.room_name.clone();

    // Fetch devices on mount
    {
        let devices = devices.clone();
        let error = error.clone();
        let room = room_name.clone();
        use_effect_with(room.clone(), move |_| {
            spawn_local(async move {
                match api::fetch_devices(&room).await {
                    Ok(data) => devices.set(data),
                    Err(e) => error.set(Some(e)),
                }
            });
        });
    }

    let on_add_device = {
        let new_device = new_device.clone();
        let device_type = device_type.clone();
        let devices = devices.clone();
        let error = error.clone();
        let room = room_name.clone();
        Callback::from(move |_| {
            let name = (*new_device).clone();
            if name.is_empty() { return; }

            let dtype = match (*device_type).as_str() {
                "thermometer" => DeviceTypeRequest::Thermometer { temperature: 20.0 },
                _ => DeviceTypeRequest::Outlet { description: "New outlet".to_string() },
            };

            let new_device = new_device.clone();
            let devices = devices.clone();
            let error = error.clone();
            let room = room.clone();
            spawn_local(async move {
                match api::create_device(&room, &name, dtype).await {
                    Ok(_) => {
                        new_device.set(String::new());
                        if let Ok(data) = api::fetch_devices(&room).await {
                            devices.set(data);
                        }
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_delete_device = {
        let devices = devices.clone();
        let error = error.clone();
        let room = room_name.clone();
        Callback::from(move |name: String| {
            let devices = devices.clone();
            let error = error.clone();
            let room = room.clone();
            spawn_local(async move {
                match api::delete_device(&room, &name).await {
                    Ok(_) => {
                        if let Ok(data) = api::fetch_devices(&room).await {
                            devices.set(data);
                        }
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_name_input = {
        let new_device = new_device.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            new_device.set(input.value());
        })
    };

    let on_type_change = {
        let device_type = device_type.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            device_type.set(select.value());
        })
    };

    html! {
        <div>
            <button class="back-btn" onclick={props.on_back.reform(|_| ())}>
                {"← Back"}
            </button>

            <h1>{"Room: "}{&props.room_name}</h1>

            if let Some(err) = (*error).clone() {
                <div class="error">{err}</div>
            }

            <div>
                <input
                    type="text"
                    placeholder="Device name"
                    value={(*new_device).clone()}
                    oninput={on_name_input}
                />
                <select onchange={on_type_change}>
                    <option value="outlet">{"Outlet"}</option>
                    <option value="thermometer">{"Thermometer"}</option>
                </select>
                <button onclick={on_add_device}>{"Add Device"}</button>
            </div>

            <h2>{"Devices"}</h2>
            <ul class="device-list">
                { for (*devices).iter().map(|device| {
                    let name = device.name.clone();
                    let on_select = props.on_select_device.reform(move |_| name.clone());
                    let name_del = device.name.clone();
                    let on_delete = on_delete_device.reform(move |_| name_del.clone());
                    html! {
                        <li class="device-item">
                            <span onclick={on_select} style="cursor: pointer;">
                                {&device.name} {" ("}{&device.device_type}{"): "}{&device.status}
                            </span>
                            <button onclick={on_delete}>{"Delete"}</button>
                        </li>
                    }
                })}
            </ul>
        </div>
    }
}
```

### 6. Device Component (`src/components/device.rs`)

```rust
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub room_name: String,
    pub device_name: String,
    pub on_back: Callback<()>,
}

#[function_component(Device)]
pub fn device(props: &Props) -> Html {
    html! {
        <div>
            <button class="back-btn" onclick={props.on_back.reform(|_| ())}>
                {"← Back"}
            </button>

            <h1>{"Device: "}{&props.device_name}</h1>
            <p>{"Room: "}{&props.room_name}</p>
        </div>
    }
}
```

### 7. Report Component (`src/components/report.rs`)

```rust
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_back: Callback<()>,
}

#[function_component(Report)]
pub fn report(props: &Props) -> Html {
    let report = use_state(String::new);
    let error = use_state(|| None::<String>);

    {
        let report = report.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::fetch_report().await {
                    Ok(data) => report.set(data),
                    Err(e) => error.set(Some(e)),
                }
            });
        });
    }

    html! {
        <div>
            <button class="back-btn" onclick={props.on_back.reform(|_| ())}>
                {"← Back"}
            </button>

            <h1>{"Home Report"}</h1>

            if let Some(err) = (*error).clone() {
                <div class="error">{err}</div>
            }

            <div class="report">
                {&*report}
            </div>
        </div>
    }
}
```

### 8. Main Entry Point (`src/main.rs`)

```rust
mod api;
mod models;
mod components;

use yew::prelude::*;
use components::{Home, Room, Device, Report};

#[derive(Clone, PartialEq)]
enum View {
    Home,
    Room(String),
    Device { room: String, device: String },
    Report,
}

#[function_component(App)]
fn app() -> Html {
    let view = use_state(|| View::Home);

    let on_select_room = {
        let view = view.clone();
        Callback::from(move |name: String| view.set(View::Room(name)))
    };

    let on_select_device = {
        let view = view.clone();
        let current_view = (*view).clone();
        Callback::from(move |device: String| {
            if let View::Room(ref room) = current_view {
                view.set(View::Device { room: room.clone(), device });
            }
        })
    };

    let on_show_report = {
        let view = view.clone();
        Callback::from(move |_| view.set(View::Report))
    };

    let on_back_to_home = {
        let view = view.clone();
        Callback::from(move |_| view.set(View::Home))
    };

    let on_back_to_room = {
        let view = view.clone();
        let current_view = (*view).clone();
        Callback::from(move |_| {
            if let View::Device { ref room, .. } = current_view {
                view.set(View::Room(room.clone()));
            }
        })
    };

    match (*view).clone() {
        View::Home => html! {
            <Home
                on_select_room={on_select_room}
                on_show_report={on_show_report}
            />
        },
        View::Room(name) => html! {
            <Room
                room_name={name}
                on_back={on_back_to_home}
                on_select_device={on_select_device}
            />
        },
        View::Device { room, device } => html! {
            <Device
                room_name={room}
                device_name={device}
                on_back={on_back_to_room}
            />
        },
        View::Report => html! {
            <Report on_back={on_back_to_home} />
        },
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

---

## Running the Frontend

### Install Trunk

```bash
cargo install trunk
```

### Run Development Server

```bash
cd web
trunk serve
```

Frontend: http://localhost:8080
Backend: http://localhost:3000 (must be running)

### Build for Production

```bash
trunk build --release
```

Output in `web/dist/`

---

## CORS Configuration (Backend)

Add CORS support to the backend to allow frontend requests:

```rust
// In server/src/main.rs
use tower_http::cors::{CorsLayer, Any};

let app = api::routes::create_router(state)
    .layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any))
    .layer(TraceLayer::new_for_http());
```

---

## Summary

| View | Features |
|------|----------|
| Home | List rooms, add room, delete room, link to report |
| Room | List devices, add device, delete device, back to home |
| Device | Show device info, back to room |
| Report | Display full home report |

This is a minimal implementation with ~400 lines of Rust code covering all required features.
