use std::net::SocketAddr;
use reqwest::Client;
use serde_json::json;
use server::app;



#[tokio::test]
async fn test_add_and_list_rooms() {
    const BASE_URL: &str = "http://localhost:8888/api/v1";
    app::spawn_test_server(SocketAddr::from(([127, 0, 0, 1], 8888))).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let client = Client::new();

    // Add a room
    let response = client
        .post(format!("{}/room/add_room", BASE_URL))
        .json(&json!({ "name": "Living Room" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // List rooms
    let response = client
        .get(format!("{}/room/room_list", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["rooms"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_add_and_get_device() {
    const BASE_URL: &str = "http://localhost:8887/api/v1";
    app::spawn_test_server(SocketAddr::from(([127, 0, 0, 1], 8887))).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let client = Client::new();

    // Create room first
    let response = client
        .post(format!("{}/room/add_room", BASE_URL))
        .json(&json!({ "name": "Kitchen" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Create device
    let response = client
        .post(format!("{}/room/Kitchen/device", BASE_URL))
        .json(&json!({
            "name": "Main Outlet",
            "device_type": {
                "type": "outlet",
                "params": { "power": 120 }
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Get device
    let response = client
        .get(format!("{}/room/Kitchen/device/Main%20Outlet/report", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_home_report() {
    const BASE_URL: &str = "http://localhost:8886/api/v1";
    app::spawn_test_server(SocketAddr::from(([127, 0, 0, 1], 8886))).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/home/report", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["report"].is_string());
}

#[tokio::test]
async fn test_delete_room() {
    const BASE_URL: &str = "http://localhost:8885/api/v1";
    app::spawn_test_server(SocketAddr::from(([127, 0, 0, 1], 8885))).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let client = Client::new();

    // Create room
    client
        .post(format!("{}/room/add_room", BASE_URL))
        .json(&json!({ "name": "Temp Room" }))
        .send()
        .await
        .unwrap();

    // Delete room
    let response = client
        .delete(format!("{}/room/Temp%20Room", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // Verify room is deleted
    let response = client
        .get(format!("{}/room/Temp%20Room", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn test_room_not_found() {
    const BASE_URL: &str = "http://localhost:8884/api/v1";
    app::spawn_test_server(SocketAddr::from(([127, 0, 0, 1], 8884))).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let client = Client::new();

    let response = client
        .get(format!("{}/room/NonExistent", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}