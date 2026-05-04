use leptos::{ev, prelude::*};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};

use crate::app::{AppState, EventTone};

async fn sleep_ms(milliseconds: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let callback = Closure::once_into_js(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });

        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.unchecked_ref(),
                milliseconds,
            );
        }
    });

    let _ = JsFuture::from(promise).await;
}

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

            <button class="feed-now-button setup-main-action" on:click=move |_| on_add()>
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
                    <button class="setup-card" on:click=move |_| on_bluetooth()>
                        <div>
                            <p class="setup-card-title">"Bluetooth"</p>
                            <p class="setup-card-copy">"Scan nearby devices and configure the feeder."</p>
                        </div>
                        <span class="settings-arrow">">"</span>
                    </button>
                    <button class="setup-card" on:click=move |_| on_wifi()>
                        <div>
                            <p class="setup-card-title">"Wi-Fi"</p>
                            <p class="setup-card-copy">"Connect directly with server address and auth token."</p>
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

    spawn_local(async move {
        sleep_ms(1_200).await;
        set_is_scanning.set(false);
    });

    let devices = || {
        vec![
            ("Barsik Feeder BLE", "-42 dBm"),
            ("Kitchen Feeder C6", "-61 dBm"),
            ("Unknown Feeder", "-77 dBm"),
        ]
    };

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
                    <div class="setup-list">
                        <For
                            each=devices
                            key=|device| device.0
                            children=move |device| {
                                view! {
                                    <button
                                        class="setup-card"
                                        on:click=move |_| {
                                            set_selected_device.set(String::from(device.0));
                                            web_sys::console::log_1(&format!("selected bluetooth device: {}", device.0).into());
                                        }
                                    >
                                        <div>
                                            <p class="setup-card-title">{device.0}</p>
                                            <p class="setup-card-copy">{device.1}</p>
                                        </div>
                                        <span class="pill-badge active">"BLE"</span>
                                    </button>
                                }
                            }
                        />
                    </div>

                    <Show when=move || !selected_device.get().is_empty()>
                        <p class="inline-status">{move || format!("Selected: {}", selected_device.get())}</p>
                        <button class="feed-now-button setup-main-action" on:click=move |_| on_continue()>
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
                <p class="panel-title">"Configure Wi-Fi and server?"</p>
                <p class="panel-subtitle">
                    "The feeder can receive Wi-Fi credentials and backend settings after Bluetooth pairing."
                </p>
                <div class="success-actions">
                    <button class="cta-button cta-primary" on:click=move |_| on_yes()>
                        <span class="cta-title">"Configure"</span>
                        <span class="cta-copy">"Scan networks and set backend access."</span>
                    </button>
                    <button class="cta-button cta-secondary" on:click=move |_| on_skip()>
                        <span class="cta-title">"Skip"</span>
                        <span class="cta-copy">"Open feeder controls now."</span>
                    </button>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn WifiSetupPage<F1, F2>(
    set_app_state: WriteSignal<AppState>,
    on_back: F1,
    on_connected: F2,
) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
{
    let (is_scanning, set_is_scanning) = signal(true);
    let (selected_network, set_selected_network) = signal(String::new());
    let (connection_type, set_connection_type) = signal(String::from("DHCP"));
    let (password, set_password) = signal(String::new());

    spawn_local(async move {
        sleep_ms(1_200).await;
        set_is_scanning.set(false);
    });

    let networks = || {
        vec![
            ("Home Wi-Fi", "-38 dBm", true),
            ("FIIT Lab", "-56 dBm", true),
            ("Open Setup Net", "-71 dBm", false),
        ]
    };

    view! {
        <section class="page">
            <SetupTopBar title="Wi-Fi setup" on_back=on_back />

            <div class="panel panel-tight">
                <Show
                    when=move || !is_scanning.get()
                    fallback=move || view! { <SetupLoader label="Scanning Wi-Fi networks..." /> }
                >
                    <div class="setup-list">
                        <For
                            each=networks
                            key=|network| network.0
                            children=move |network| {
                                view! {
                                    <button
                                        class="setup-card"
                                        on:click=move |_| set_selected_network.set(String::from(network.0))
                                    >
                                        <div>
                                            <p class="setup-card-title">{network.0}</p>
                                            <p class="setup-card-copy">{network.1}</p>
                                        </div>
                                        <span class=if network.2 { "pill-badge inactive" } else { "pill-badge active" }>
                                            {if network.2 { "Locked" } else { "Open" }}
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
                                <option value="Static IPv4">"Static IPv4"</option>
                                <option value="DHCPv6">"DHCPv6"</option>
                                <option value="Static IPv6">"Static IPv6"</option>
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
                                set_app_state.update(|state| {
                                    state.push_event(
                                        String::from("Wi-Fi configured"),
                                        format!("{} using {}", selected_network.get(), connection_type.get()),
                                        String::from("Now"),
                                        EventTone::Success,
                                    );
                                });
                                on_connected();
                            }
                        >
                            "Connect"
                        </button>
                        <button class="text-button setup-cancel" on:click=move |_| set_selected_network.set(String::new())>
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Show>
        </section>
    }
}

#[component]
pub fn ServerSetupPage<F1, F2>(
    set_app_state: WriteSignal<AppState>,
    on_back: F1,
    on_done: F2,
) -> impl IntoView
where
    F1: Fn() + Copy + Send + Sync + 'static,
    F2: Fn() + Copy + Send + Sync + 'static,
{
    let (server_url, set_server_url) = signal(String::from("http://127.0.0.1:8081"));
    let (token, set_token) = signal(String::new());

    view! {
        <section class="page">
            <SetupTopBar title="Server access" on_back=on_back />

            <div class="panel panel-tight">
                <p class="panel-title">"Backend connection"</p>
                <p class="panel-subtitle">
                    "Enter server URL and authentication token. This screen is ready for the future real request."
                </p>

                <label class="settings-field">
                    <span class="settings-label">"Server URL"</span>
                    <input
                        class="settings-input"
                        prop:value=move || server_url.get()
                        on:input=move |event| set_server_url.set(event_target_value(&event))
                    />
                </label>

                <label class="settings-field">
                    <span class="settings-label">"Auth token"</span>
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
                        let feeder_id = set_app_state
                            .try_update(|state| {
                                let id = state.add_demo_feeder(
                                    String::from("New configured feeder"),
                                    String::from("Wi-Fi"),
                                );
                                state.select_feeder(id);
                                state.push_event(
                                    String::from("Server connected"),
                                    server_url.get(),
                                    String::from("Now"),
                                    EventTone::Success,
                                );
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
            <button class="back-button" on:click=move |_| on_back()>
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
