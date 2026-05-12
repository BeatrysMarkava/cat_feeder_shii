use serde::Deserialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct BleDevice {
    pub id: String,
    pub name: String,
    pub signal_strength: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: i32,
    pub locked: bool,
    pub network_index: usize,
}

fn tauri_invoke_function() -> Result<js_sys::Function, String> {
    let window = web_sys::window().ok_or_else(|| String::from("App window is unavailable"))?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| String::from("Device connection is unavailable"))?;

    if tauri.is_undefined() || tauri.is_null() {
        return Err(String::from("Device connection is unavailable"));
    }

    let core = js_sys::Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| String::from("Device connection is unavailable"))?;
    let invoke = js_sys::Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| String::from("Device connection is unavailable"))?;

    invoke
        .dyn_into::<js_sys::Function>()
        .map_err(|_| String::from("Device connection is unavailable"))
}

async fn invoke(command: &str, args: Option<js_sys::Object>) -> Result<JsValue, String> {
    let invoke = tauri_invoke_function()?;
    let args = args.unwrap_or_else(js_sys::Object::new);
    let promise = invoke
        .call2(&JsValue::NULL, &JsValue::from_str(command), &args)
        .map_err(|_| String::from("Request failed"))?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| String::from("Request could not be started"))?;

    JsFuture::from(promise)
        .await
        .map_err(|_| String::from("Request could not be completed"))
}

fn set_arg(args: &js_sys::Object, key: &str, value: &JsValue) -> Result<(), String> {
    js_sys::Reflect::set(args, &JsValue::from_str(key), value)
        .map(|_| ())
        .map_err(|_| format!("failed to set argument {key}"))
}

fn parse_list<T>(value: JsValue) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let json = js_sys::JSON::stringify(&value)
        .map_err(|_| String::from("Could not read response"))?
        .as_string()
        .ok_or_else(|| String::from("Unexpected response format"))?;

    serde_json::from_str(&json).map_err(|error| error.to_string())
}

pub async fn frontend_action(action: &str, detail: Option<String>) -> Result<(), String> {
    let args = js_sys::Object::new();
    set_arg(&args, "action", &JsValue::from_str(action))?;
    if let Some(detail) = detail {
        set_arg(&args, "detail", &JsValue::from_str(&detail))?;
    }

    invoke("frontend_action", Some(args)).await.map(|_| ())
}

pub fn report_button_click(action: &'static str, detail: Option<String>) {
    spawn_local(async move {
        let _ = frontend_action(action, detail).await;
    });
}

pub async fn scan_ble_devices() -> Result<Vec<BleDevice>, String> {
    invoke("scan_ble_devices", None).await.and_then(parse_list)
}

pub async fn connect_esp32c6(device_id: &str) -> Result<(), String> {
    let args = js_sys::Object::new();
    set_arg(&args, "deviceId", &JsValue::from_str(device_id))?;
    invoke("connect_esp32c6", Some(args)).await.map(|_| ())
}

pub async fn scan_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    invoke("scan_wifi_networks", None)
        .await
        .and_then(parse_list)
}

pub async fn send_wifi_credentials(
    ssid: &str,
    network_index: usize,
    password: &str,
    connection_type: &str,
) -> Result<String, String> {
    let credentials = js_sys::Object::new();
    set_arg(&credentials, "ssid", &JsValue::from_str(ssid))?;
    set_arg(&credentials, "password", &JsValue::from_str(password))?;
    set_arg(
        &credentials,
        "network_index",
        &JsValue::from_f64(network_index as f64),
    )?;
    set_arg(
        &credentials,
        "connection_type",
        &JsValue::from_str(connection_type),
    )?;

    let args = js_sys::Object::new();
    set_arg(&args, "credentials", credentials.as_ref())?;

    let response = invoke("send_wifi_credentials", Some(args)).await?;
    response
        .as_string()
        .ok_or_else(|| String::from("Unexpected Wi-Fi setup response"))
}

pub async fn feed_now_via_tauri(portion: u8) -> Result<String, String> {
    let args = js_sys::Object::new();
    set_arg(&args, "portion", &JsValue::from_f64(f64::from(portion)))?;
    let response = invoke("feed_now_via_tauri", Some(args)).await?;

    response
        .as_string()
        .ok_or_else(|| String::from("Unexpected feeding response"))
}
