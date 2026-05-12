use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BleDevice {
    pub id: String,
    pub name: String,
    pub signal_strength: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WiFiCredentials {
    pub ssid: String,
    pub password: String,
    pub network_index: usize,
    pub connection_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: i32,
    pub locked: bool,
    pub network_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub is_connected: bool,
    pub wifi_ssid: Option<String>,
    pub signal_strength: i32,
    pub battery_level: u8,
    pub last_sync: Option<String>,
}

#[tauri::command]
pub async fn scan_ble_devices() -> Result<Vec<BleDevice>, String> {
    println!("Starting BLE device scan...");

    match crate::ble::scan_devices().await {
        Ok(devices) => {
            println!("Found {} devices", devices.len());
            Ok(devices)
        }
        Err(e) => {
            eprintln!("BLE scan error: {}", e);
            Err(format!("Failed to scan BLE devices: {}", e))
        }
    }
}

#[tauri::command]
pub fn frontend_action(action: String, detail: Option<String>) -> Result<(), String> {
    println!(
        "Frontend action: action={}, detail={}",
        action,
        detail.unwrap_or_else(|| "-".to_string())
    );
    Ok(())
}

#[tauri::command]
pub async fn scan_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    println!("Starting ESP32C6 WiFi network scan over BLE...");

    match crate::ble::scan_wifi_networks().await {
        Ok(networks) => {
            println!("Found {} WiFi network(s) through ESP32C6 BLE", networks.len());
            Ok(networks)
        }
        Err(error) => {
            eprintln!("ESP32C6 WiFi scan failed: {}", error);
            Err(format!("Failed to scan WiFi through ESP32C6 BLE: {}", error))
        }
    }
}

#[tauri::command]
pub async fn feed_now_via_tauri(portion: u8) -> Result<String, String> {
    println!("Feed Now requested through Tauri API: portion={}", portion);

    if !(1..=5).contains(&portion) {
        return Err("portion must be between 1 and 5".to_string());
    }

    let connected_devices = crate::ble::connected_device_ids().await;
    if connected_devices.is_empty() {
        println!("Feed Now cannot be sent: no BLE device is connected");
        return Err("No BLE device is connected in Tauri".to_string());
    }

    println!(
        "Feed Now reached Tauri. Connected BLE devices: {}. Real BLE feed write is not implemented yet.",
        connected_devices.join(", ")
    );
    Err(String::from(
        "Direct BLE feeding is not implemented in Tauri yet; use the backend command queue.",
    ))
}

#[tauri::command]
pub async fn connect_esp32c6(device_id: String) -> Result<String, String> {
    println!("Connecting to ESP32C6: {}", device_id);

    match crate::ble::connect_device(&device_id).await {
        Ok(_) => Ok(format!("Successfully connected to {}", device_id)),
        Err(e) => {
            eprintln!("Connection error: {}", e);
            Err(format!("Failed to connect: {}", e))
        }
    }
}

#[tauri::command]
pub async fn disconnect_esp32c6(device_id: String) -> Result<String, String> {
    println!("Disconnecting from ESP32C6: {}", device_id);

    match crate::ble::disconnect_device(&device_id).await {
        Ok(_) => Ok(format!("Disconnected from {}", device_id)),
        Err(e) => {
            eprintln!("Disconnection error: {}", e);
            Err(format!("Failed to disconnect: {}", e))
        }
    }
}

#[tauri::command]
pub async fn send_wifi_credentials(
    credentials: WiFiCredentials,
) -> Result<String, String> {
    println!(
        "Sending WiFi credentials to selected ESP32C6 network: ssid={}, index={}",
        credentials.ssid, credentials.network_index
    );

    match crate::ble::provision_wifi(&credentials).await {
        Ok(_) => Ok("WiFi credentials sent successfully".to_string()),
        Err(e) => {
            eprintln!("Provisioning error: {}", e);
            Err(format!("Failed to send credentials: {}", e))
        }
    }
}

#[tauri::command]
pub fn get_device_status(device_id: String) -> Result<DeviceStatus, String> {
    println!("Getting status for device: {}", device_id);

    Ok(DeviceStatus {
        device_id: device_id.clone(),
        is_connected: false,
        wifi_ssid: None,
        signal_strength: -80,
        battery_level: 75,
        last_sync: None,
    })
}

#[tauri::command]
pub async fn health_check() -> Result<String, String> {
    match reqwest::Client::new()
        .get("http://localhost:8000/health")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok("Backend server is healthy".to_string())
            } else {
                Err("Backend server returned error".to_string())
            }
        }
        Err(e) => Err(format!("Health check failed: {}", e)),
    }
}
