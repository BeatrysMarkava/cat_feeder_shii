use leptos::{ev, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::app::AppState;
use crate::tauri_api::{self, BleDevice, WifiNetwork};

#[component]
pub fn FeederListPage<F1, F2>(
    app_state: ReadSignal<AppState>,
    set_app_state: WriteSignal<AppState>,
    on_add: F1,
    on_open: F2,
) -> impl IntoView
where
    F1: Fn() + Copy + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <section class="page page-home">
            <div class="hero-card hero-card-single">
                <div class="hero-copy">
                    <p class="eyebrow">"Smart cat feeder"</p>
                    <h1 class="hero-title">"Feeders"</h1>
                    <p class="hero-subtitle">
                        "Choose a feeder to control, or add a new one."
                    </p>
                </div>
            </div>

            <div class="setup-list">
                <For
                    each=move || app_state.get().feeders
                    key=|feeder| feeder.id
                    children=move |feeder| {
                        let feeder_id = feeder.id;
                        let feeder_status = feeder.status.clone();
                        let feeder_status_class = if feeder_status == "Online" {
                            "pill-badge active"
                        } else {
                            "pill-badge inactive"
                        };
                        view! {
                            <button
                                class="setup-card"
                                on:click=move |_| {
                                    tauri_api::report_button_click(
                                        "feeder_card_clicked",
                                        Some(format!("id={feeder_id}")),
                                    );
                                    set_app_state.update(|state| state.select_feeder(feeder_id));
                                    on_open();
                                }
                            >
                                <div>
                                    <p class="setup-card-title">{feeder.name}</p>
                                    <p class="setup-card-copy">
                                        {format!("{} - {}% food", feeder.connection, feeder.hopper_level)}
                                    </p>
                                </div>
                                <span class=feeder_status_class>
                                    {feeder_status}
                                </span>
                            </button>
                        }
                    }
                />
            </div>

            <button
                class="feed-now-button setup-main-action"
                on:click=move |_| {
                    tauri_api::report_button_click("add_new_feeder_clicked", None);
                    on_add();
                }
            >
                "Add new feeder"
            </button>
        </section>
    }
}

#[component]
pub fn AddFeederPage<F1, F2, F3>(on_back: F1, on_bluetooth: F2, on_wifi: F3) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
    F3: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <section class="page">
            <SetupTopBar title="Add feeder" on_back=on_back />

            <div class="panel panel-tight">
                <div class="panel-header">
                    <div>
                        <p class="panel-title">"Connection method"</p>
                        <p class="panel-subtitle">
                            "Choose how this app should discover or connect to the feeder."
                        </p>
                    </div>
                </div>

                <div class="setup-list">
                    <button
                        class="setup-card"
                        on:click=move |_| {
                            tauri_api::report_button_click("add_feeder_bluetooth_clicked", None);
                            on_bluetooth();
                        }
                    >
                        <div>
                            <p class="setup-card-title">"Bluetooth"</p>
                            <p class="setup-card-copy">"Scan nearby devices and configure the feeder."</p>
                        </div>
                        <span class="settings-arrow">">"</span>
                    </button>
                    <button
                        class="setup-card"
                        on:click=move |_| {
                            tauri_api::report_button_click("add_feeder_wifi_clicked", None);
                            on_wifi();
                        }
                    >
                        <div>
                            <p class="setup-card-title">"Wi-Fi"</p>
                            <p class="setup-card-copy">"Finish setup using your feeder connection details."</p>
                        </div>
                        <span class="settings-arrow">">"</span>
                    </button>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn BluetoothScanPage<F1, F2>(on_back: F1, on_continue: F2) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
{
    let (is_scanning, set_is_scanning) = signal(true);
    let (selected_device, set_selected_device) = signal(String::new());
    let (selected_device_id, set_selected_device_id) = signal(String::new());
    let (is_connected, set_is_connected) = signal(false);
    let (devices, set_devices) = signal(Vec::<BleDevice>::new());
    let (scan_status, set_scan_status) = signal(String::new());

    spawn_local(async move {
        let _ = tauri_api::frontend_action("bluetooth_scan_opened", None).await;
        match tauri_api::scan_ble_devices().await {
            Ok(found_devices) => {
                set_scan_status.set(format!("Found {} device(s).", found_devices.len()));
                set_devices.set(found_devices);
            }
            Err(error) => {
                set_scan_status.set(format!("Bluetooth scan unavailable: {error}"));
                set_devices.set(Vec::new());
            }
        }
        set_is_scanning.set(false);
    });

    view! {
        <section class="page">
            <SetupTopBar title="Bluetooth scan" on_back=on_back />

            <div class="panel panel-tight">
                <div class="panel-header">
                    <div>
                        <p class="panel-title">"Nearby devices"</p>
                        <p class="panel-subtitle">"Select a feeder to continue setup."</p>
                    </div>
                </div>

                <Show
                    when=move || !is_scanning.get()
                    fallback=move || view! { <SetupLoader label="Scanning Bluetooth devices..." /> }
                >
                    <Show when=move || !scan_status.get().is_empty()>
                        <p class="inline-status">{move || scan_status.get()}</p>
                    </Show>

                    <div class="setup-list">
                        <For
                            each=move || devices.get()
                            key=|device| device.id.clone()
                            children=move |device| {
                                let device_id = device.id.clone();
                                let device_name = device.name.clone();
                                let signal_strength = device.signal_strength;
                                view! {
                                    <button
                                        class="setup-card"
                                        on:click=move |_| {
                                            set_selected_device.set(device_name.clone());
                                            set_selected_device_id.set(device_id.clone());
                                            set_is_connected.set(false);
                                            set_scan_status.set(format!("Connecting to {}...", device_name));
                                            spawn_local({
                                                let device_id = device_id.clone();
                                                let device_name = device_name.clone();
                                                async move {
                                                    let _ = tauri_api::frontend_action(
                                                        "bluetooth_device_clicked",
                                                        Some(format!("id={}, name={}", device_id, device_name)),
                                                    )
                                                    .await;

                                                    match tauri_api::connect_esp32c6(&device_id).await {
                                                        Ok(()) => {
                                                            set_is_connected.set(true);
                                                            set_scan_status.set(format!("Connected to {}.", device_name));
                                                        }
                                                        Err(error) => {
                                                            set_is_connected.set(false);
                                                            set_scan_status.set(format!("Connection failed: {error}"));
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    >
                                        <div>
                                            <p class="setup-card-title">{device.name}</p>
                                            <p class="setup-card-copy">{format!("{} dBm", signal_strength)}</p>
                                        </div>
                                        <span class="pill-badge active">"Bluetooth"</span>
                                    </button>
                                }
                            }
                        />
                    </div>

                    <Show when=move || !selected_device.get().is_empty() && is_connected.get()>
                        <p class="inline-status">{move || format!("Selected: {}", selected_device.get())}</p>
                        <button
                            class="feed-now-button setup-main-action"
                            on:click=move |_| {
                                spawn_local(async move {
                                    let _ = tauri_api::frontend_action(
                                        "bluetooth_continue_clicked",
                                        Some(format!("device_id={}", selected_device_id.get())),
                                    )
                                    .await;
                                });
                                on_continue();
                            }
                        >
                            "Continue"
                        </button>
                    </Show>
                </Show>
            </div>
        </section>
    }
}

#[component]
pub fn WifiSetupQuestionPage<F1, F2, F3>(on_back: F1, on_yes: F2, on_skip: F3) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
    F3: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <section class="page">
            <SetupTopBar title="Network setup" on_back=on_back />
            <div class="panel panel-tight">
                <p class="panel-title">"Configure network access?"</p>
                <p class="panel-subtitle">
                    "Send network details to the feeder after Bluetooth pairing."
                </p>
                <div class="success-actions">
                    <button
                        class="cta-button cta-primary"
                        on:click=move |_| {
                            tauri_api::report_button_click("wifi_setup_question_configure_clicked", None);
                            on_yes();
                        }
                    >
                        <span class="cta-title">"Configure"</span>
                        <span class="cta-copy">"Scan networks and connect the feeder."</span>
                    </button>
                    <button
                        class="cta-button cta-secondary"
                        on:click=move |_| {
                            tauri_api::report_button_click("wifi_setup_question_skip_clicked", None);
                            on_skip();
                        }
                    >
                        <span class="cta-title">"Skip"</span>
                        <span class="cta-copy">"Open feeder controls now."</span>
                    </button>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn WifiSetupPage<F1, F2>(on_back: F1, on_connected: F2) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
{
    let (is_scanning, set_is_scanning) = signal(true);
    let (selected_network, set_selected_network) = signal(String::new());
    let (selected_network_index, set_selected_network_index) = signal(0usize);
    let (connection_type, set_connection_type) = signal(String::from("DHCP"));
    let (password, set_password) = signal(String::new());
    let (networks, set_networks) = signal(Vec::<WifiNetwork>::new());
    let (scan_status, set_scan_status) = signal(String::new());

    spawn_local(async move {
        let _ = tauri_api::frontend_action("wifi_scan_opened", None).await;
        match tauri_api::scan_wifi_networks().await {
            Ok(found_networks) => {
                set_scan_status.set(format!("Found {} network(s).", found_networks.len()));
                set_networks.set(found_networks);
            }
            Err(error) => {
                set_scan_status.set(format!("Wi-Fi scan unavailable: {error}"));
                set_networks.set(Vec::new());
            }
        }
        set_is_scanning.set(false);
    });

    view! {
        <section class="page">
            <SetupTopBar title="Wi-Fi setup" on_back=on_back />

            <div class="panel panel-tight">
                <Show
                    when=move || !is_scanning.get()
                    fallback=move || view! { <SetupLoader label="Scanning Wi-Fi networks..." /> }
                >
                    <Show when=move || !scan_status.get().is_empty()>
                        <p class="inline-status">{move || scan_status.get()}</p>
                    </Show>

                    <div class="setup-list">
                        <For
                            each=move || networks.get()
                            key=|network| network.ssid.clone()
                            children=move |network| {
                                let ssid = network.ssid.clone();
                                let network_index = network.network_index;
                                let signal_strength = network.signal_strength;
                                let locked = network.locked;
                                view! {
                                    <button
                                        class="setup-card"
                                        on:click=move |_| {
                                            set_selected_network.set(ssid.clone());
                                            set_selected_network_index.set(network_index);
                                            spawn_local({
                                                let ssid = ssid.clone();
                                                async move {
                                                    let _ = tauri_api::frontend_action(
                                                        "wifi_network_clicked",
                                                        Some(format!("ssid={ssid}")),
                                                    )
                                                    .await;
                                                }
                                            });
                                        }
                                    >
                                        <div>
                                            <p class="setup-card-title">{network.ssid}</p>
                                            <p class="setup-card-copy">{format!("{} dBm", signal_strength)}</p>
                                        </div>
                                        <span class=if locked { "pill-badge inactive" } else { "pill-badge active" }>
                                            {if locked { "Locked" } else { "Open" }}
                                        </span>
                                    </button>
                                }
                            }
                        />
                    </div>
                </Show>
            </div>

            <Show when=move || !selected_network.get().is_empty()>
                <div class="setup-modal-backdrop">
                    <div class="setup-modal">
                        <p class="panel-title">{move || selected_network.get()}</p>
                        <label class="settings-field">
                            <span class="settings-label">"Connection type"</span>
                            <select
                                class="settings-input"
                                prop:value=move || connection_type.get()
                                on:change=move |event: ev::Event| {
                                    set_connection_type.set(event_target_value(&event));
                                }
                            >
                                <option value="DHCP">"DHCP"</option>
                                <option value="DHCPv6">"DHCPv6"</option>
                            </select>
                        </label>
                        <label class="settings-field">
                            <span class="settings-label">"Password"</span>
                            <input
                                class="settings-input"
                                type="password"
                                prop:value=move || password.get()
                                on:input=move |event| set_password.set(event_target_value(&event))
                            />
                        </label>
                        <button
                            class="feed-now-button"
                            on:click=move |_| {
                                set_scan_status.set(format!(
                                    "Sending credentials for {}...",
                                    selected_network.get()
                                ));
                                spawn_local(async move {
                                    let _ = tauri_api::frontend_action(
                                        "wifi_connect_clicked",
                                        Some(format!(
                                            "ssid={}, network_index={}, connection_type={}",
                                            selected_network.get(),
                                            selected_network_index.get(),
                                            connection_type.get()
                                        )),
                                    )
                                    .await;

                                    match tauri_api::send_wifi_credentials(
                                        &selected_network.get(),
                                        selected_network_index.get(),
                                        &password.get(),
                                        &connection_type.get(),
                                    )
                                    .await
                                    {
                                        Ok(message) => {
                                            set_scan_status.set(message.clone());
                                            let _ = tauri_api::frontend_action(
                                                "wifi_credentials_sent",
                                                Some(message),
                                            )
                                            .await;
                                            on_connected();
                                        }
                                        Err(error) => {
                                            set_scan_status.set(format!(
                                                "Wi-Fi provisioning failed: {error}"
                                            ));
                                            let _ = tauri_api::frontend_action(
                                                "wifi_credentials_error",
                                                Some(error.clone()),
                                            )
                                            .await;
                                            web_sys::console::error_1(
                                                &format!("Wi-Fi provisioning failed: {error}").into(),
                                            );
                                        }
                                    }
                                });
                            }
                        >
                            "Connect"
                        </button>
                        <button
                            class="text-button setup-cancel"
                            on:click=move |_| {
                                tauri_api::report_button_click("wifi_modal_cancel_clicked", None);
                                set_selected_network.set(String::new());
                            }
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Show>
        </section>
    }
}

#[component]
pub fn ConnectionDetailsPage<F1, F2>(
    set_app_state: WriteSignal<AppState>,
    on_back: F1,
    on_done: F2,
) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
{
    let (connection_address, set_connection_address) = signal(String::from("http://127.0.0.1:8081"));
    let (token, set_token) = signal(String::new());

    view! {
        <section class="page">
            <SetupTopBar title="Connection details" on_back=on_back />

            <div class="panel panel-tight">
                <p class="panel-title">"Feeder connection"</p>
                <p class="panel-subtitle">
                    "Enter the address and access key for this feeder."
                </p>

                <label class="settings-field">
                    <span class="settings-label">"Address"</span>
                    <input
                        class="settings-input"
                        prop:value=move || connection_address.get()
                        on:input=move |event| set_connection_address.set(event_target_value(&event))
                    />
                </label>

                <label class="settings-field">
                    <span class="settings-label">"Access key"</span>
                    <input
                        class="settings-input"
                        type="password"
                        prop:value=move || token.get()
                        on:input=move |event| set_token.set(event_target_value(&event))
                    />
                </label>

                <button
                    class="feed-now-button setup-main-action"
                    on:click=move |_| {
                        tauri_api::report_button_click(
                            "connection_details_connect_clicked",
                            Some(format!("address={}", connection_address.get())),
                        );
                        let feeder_id = set_app_state
                            .try_update(|state| {
                                let id = state.add_demo_feeder(
                                    String::from("New configured feeder"),
                                    String::from("Wi-Fi"),
                                );
                                state.select_feeder(id);
                                id
                            })
                            .unwrap_or(1);
                        web_sys::console::log_1(&format!("configured feeder id: {feeder_id}").into());
                        let _ = token.get();
                        on_done();
                    }
                >
                    "Connect"
                </button>
            </div>
        </section>
    }
}

#[component]
fn SetupTopBar<F>(title: &'static str, on_back: F) -> impl IntoView
where
    F: Fn() + Copy + 'static,
{
    view! {
        <div class="top-bar">
            <button
                class="back-button"
                on:click=move |_| {
                    tauri_api::report_button_click("setup_back_clicked", Some(String::from(title)));
                    on_back();
                }
            >
                "<"
            </button>
            <div class="app-title">{title}</div>
        </div>
    }
}

#[component]
fn SetupLoader(label: &'static str) -> impl IntoView {
    view! {
        <div class="setup-loader">
            <div class="setup-spinner"></div>
            <p class="panel-subtitle">{label}</p>
        </div>
    }
}
