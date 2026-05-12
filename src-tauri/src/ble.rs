use anyhow::Result;
use btleplug::api::{
    Central, Characteristic, Manager as BleManager, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Manager, Peripheral as PlatformPeripheral};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::{uuid, Uuid};

use crate::commands::{BleDevice, WiFiCredentials, WifiNetwork};

const SERVICE_UUID: Uuid = uuid!("5f435fa5-adee-4e9a-b9a3-b812d2628906");
const WIFI_SCAN_CMD: Uuid = uuid!("d88a7d46-9313-4240-915a-a2320fa3a6e5");
const WIFI_GET_STATUS: Uuid = uuid!("878b58f2-4d44-4178-ae8f-a9e56d607e9e");
const WIFI_GET_PAGES_COUNT: Uuid = uuid!("0ce70db8-be92-4160-a2d3-588e8b248b95");
const WIFI_SELECT_PAGE: Uuid = uuid!("6839a19d-8a4b-4691-89cc-7a312c1efe54");
const WIFI_GET_PAGE_DATA: Uuid = uuid!("9c0c07d7-0435-4a9d-b999-369c8f646252");
const WIFI_SET_SSID_INDEX: Uuid = uuid!("824f9460-5d76-4498-a549-0020100907bc");
const WIFI_SET_PASSWORD: Uuid = uuid!("273d7528-c072-4fe6-b29b-c1e468f039f2");
const WIFI_SET_CONNECTION_TYPE: Uuid = uuid!("25422a9b-558d-49f1-8db9-30bbfe8b1c2c");
const WIFI_CONNECT: Uuid = uuid!("2c1f2d97-5c53-435b-940c-c36cf349ca53");

const SHORT_SSID_LEN: usize = 16;
const SERIALIZED_SSID_LEN: usize = SHORT_SSID_LEN + 1 + 1;
const MAX_SSID_PER_PAGE: usize = 5;
const ENTIRE_SSID_PAGE_SIZE: usize = SERIALIZED_SSID_LEN * MAX_SSID_PER_PAGE;

lazy_static::lazy_static! {
    static ref DISCOVERED_DEVICES: Arc<Mutex<HashMap<String, PlatformPeripheral>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref CONNECTED_DEVICES: Arc<Mutex<HashMap<String, PlatformPeripheral>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum WifiStatus {
    Idle = 0,
    Scanning = 1,
    Connecting = 2,
    ScannedSuccessfully = 51,
    Connected = 52,
    ErrorWhileScanning = 201,
    ErrorWhileConnecting = 202,
    ErrorNoScannedNetworks = 206,
    Error = 255,
}

impl From<u8> for WifiStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Idle,
            1 => Self::Scanning,
            2 => Self::Connecting,
            51 => Self::ScannedSuccessfully,
            52 => Self::Connected,
            201 => Self::ErrorWhileScanning,
            202 => Self::ErrorWhileConnecting,
            206 => Self::ErrorNoScannedNetworks,
            _ => Self::Error,
        }
    }
}

pub async fn scan_devices() -> Result<Vec<BleDevice>> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    if adapters.is_empty() {
        return Ok(vec![]);
    }

    let adapter = &adapters[0];

    // Start scanning
    adapter.start_scan(ScanFilter::default()).await?;

    // Wait a bit for devices to be discovered
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Get peripherals
    let peripherals = adapter.peripherals().await?;
    let mut devices = Vec::new();
    let mut discovered_devices = DISCOVERED_DEVICES.lock().await;
    discovered_devices.clear();

    for peripheral in peripherals {
        let device_id = peripheral.id().to_string();
        discovered_devices.insert(device_id.clone(), peripheral.clone());

        if let Ok(properties) = peripheral.properties().await {
            if let Some(properties) = properties {
                let name = properties
                    .local_name
                    .or(properties.advertisement_name)
                    .unwrap_or_else(|| String::from("Unknown BLE device"));
                let signal_strength = properties
                    .rssi
                    .map(i32::from)
                    .unwrap_or_else(|| i32::from(properties.tx_power_level.unwrap_or(-100)));

                devices.push(BleDevice {
                    id: device_id,
                    name,
                    signal_strength,
                });
            }
        }
    }

    // Stop scanning
    adapter.stop_scan().await?;

    Ok(devices)
}

pub async fn connect_device(device_id: &str) -> Result<()> {
    if let Some(peripheral) = DISCOVERED_DEVICES.lock().await.get(device_id).cloned() {
        connect_peripheral(device_id, peripheral).await?;
        return Ok(());
    }

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        anyhow::bail!("No BLE adapters found");
    }

    let adapter = &adapters[0];
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let peripherals = adapter.peripherals().await?;

    for peripheral in peripherals {
        if peripheral.id().to_string() == device_id {
            connect_peripheral(device_id, peripheral).await?;
            return Ok(());
        }
    }

    anyhow::bail!("Device not found: {}", device_id)
}

async fn connect_peripheral(device_id: &str, peripheral: PlatformPeripheral) -> Result<()> {
    println!("Connecting to device: {}", device_id);

    if !peripheral.is_connected().await? {
        peripheral.connect().await?;
    }

    peripheral.discover_services().await?;
    if !peripheral
        .services()
        .iter()
        .any(|service| service.uuid == SERVICE_UUID)
    {
        anyhow::bail!("Device does not expose the ESP32C6 provisioning service");
    }

    println!("Successfully connected to {}", device_id);

    let mut devices = CONNECTED_DEVICES.lock().await;
    devices.insert(device_id.to_string(), peripheral);

    Ok(())
}

pub async fn disconnect_device(device_id: &str) -> Result<()> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;

    if adapters.is_empty() {
        anyhow::bail!("No BLE adapters found");
    }

    let adapter = &adapters[0];
    let peripherals = adapter.peripherals().await?;

    for peripheral in peripherals {
        if peripheral.id().to_string() == device_id {
            println!("Disconnecting from device: {}", device_id);
            peripheral.disconnect().await?;

            let mut devices = CONNECTED_DEVICES.lock().await;
            devices.remove(device_id);

            return Ok(());
        }
    }

    anyhow::bail!("Device not found: {}", device_id)
}

pub async fn scan_wifi_networks() -> Result<Vec<WifiNetwork>> {
    let peripheral = selected_connected_peripheral().await?;

    write_characteristic(&peripheral, WIFI_SCAN_CMD, &[1u8]).await?;
    wait_for_status(
        &peripheral,
        WifiStatus::Scanning,
        WifiStatus::ScannedSuccessfully,
    )
    .await?;

    let pages = read_characteristic(&peripheral, WIFI_GET_PAGES_COUNT)
        .await?
        .first()
        .copied()
        .unwrap_or(0);

    let mut networks = Vec::new();
    for page in 0..pages {
        write_characteristic(&peripheral, WIFI_SELECT_PAGE, &[page]).await?;
        let data = read_characteristic(&peripheral, WIFI_GET_PAGE_DATA).await?;
        networks.extend(parse_wifi_page(&data, networks.len()));
    }

    Ok(networks)
}

pub async fn provision_wifi(credentials: &WiFiCredentials) -> Result<()> {
    println!(
        "Provisioning WiFi over BLE: ssid={}, index={}, connection_type={}",
        credentials.ssid, credentials.network_index, credentials.connection_type
    );

    let peripheral = selected_connected_peripheral().await?;
    let network_index = u8::try_from(credentials.network_index)?;

    write_characteristic(&peripheral, WIFI_SET_SSID_INDEX, &[network_index]).await?;
    write_characteristic(
        &peripheral,
        WIFI_SET_CONNECTION_TYPE,
        &connection_type_bytes(&credentials.connection_type)?,
    )
    .await?;
    write_characteristic(&peripheral, WIFI_SET_PASSWORD, credentials.password.as_bytes()).await?;
    write_characteristic(&peripheral, WIFI_CONNECT, &[1u8]).await?;

    wait_for_status(&peripheral, WifiStatus::Connecting, WifiStatus::Connected).await
}

pub async fn connected_device_ids() -> Vec<String> {
    let devices = CONNECTED_DEVICES.lock().await;
    devices.keys().cloned().collect()
}

async fn selected_connected_peripheral() -> Result<PlatformPeripheral> {
    let devices = CONNECTED_DEVICES.lock().await;
    devices
        .values()
        .next()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No ESP32C6 BLE device is connected"))
}

fn find_characteristic(peripheral: &PlatformPeripheral, uuid: Uuid) -> Result<Characteristic> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|characteristic| characteristic.uuid == uuid)
        .ok_or_else(|| anyhow::anyhow!("Characteristic {} not found", uuid))
}

async fn write_characteristic(
    peripheral: &PlatformPeripheral,
    uuid: Uuid,
    value: &[u8],
) -> Result<()> {
    let characteristic = find_characteristic(peripheral, uuid)?;
    peripheral
        .write(&characteristic, value, WriteType::WithResponse)
        .await?;
    Ok(())
}

async fn read_characteristic(peripheral: &PlatformPeripheral, uuid: Uuid) -> Result<Vec<u8>> {
    let characteristic = find_characteristic(peripheral, uuid)?;
    Ok(peripheral.read(&characteristic).await?)
}

async fn wait_for_status(
    peripheral: &PlatformPeripheral,
    in_progress: WifiStatus,
    success: WifiStatus,
) -> Result<()> {
    for _ in 0..30 {
        let status = read_characteristic(peripheral, WIFI_GET_STATUS)
            .await?
            .first()
            .copied()
            .map(WifiStatus::from)
            .unwrap_or(WifiStatus::Error);

        if status == success {
            return Ok(());
        }

        if status == WifiStatus::Idle || status == in_progress {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        }

        anyhow::bail!("Unexpected WiFi status: {:?}", status);
    }

    anyhow::bail!("Timed out waiting for WiFi status {:?}", success)
}

fn parse_wifi_page(data: &[u8], start_index: usize) -> Vec<WifiNetwork> {
    let mut result = Vec::new();
    for slot in 0..MAX_SSID_PER_PAGE {
        let offset = slot * SERIALIZED_SSID_LEN;
        if offset + SERIALIZED_SSID_LEN > data.len() || offset >= ENTIRE_SSID_PAGE_SIZE {
            break;
        }

        let ssid = String::from_utf8_lossy(&data[offset..offset + SHORT_SSID_LEN])
            .trim_end_matches(&['\0', '\n'])
            .to_string();
        if ssid.is_empty() {
            break;
        }

        let signal_strength = i32::from(data[offset + SERIALIZED_SSID_LEN - 2] as i8);
        let auth_method = data[offset + SERIALIZED_SSID_LEN - 1];

        result.push(WifiNetwork {
            ssid,
            signal_strength,
            locked: auth_method != 0,
            network_index: start_index + result.len(),
        });
    }

    result
}

fn connection_type_bytes(connection_type: &str) -> Result<Vec<u8>> {
    match connection_type {
        "DHCP" => Ok(vec![0]),
        "DHCPv6" => Ok(vec![2]),
        "Static IPv4" | "Static IPv6" => {
            anyhow::bail!("Static IP provisioning is not supported from this UI yet")
        }
        other => anyhow::bail!("Unsupported connection type: {}", other),
    }
}
