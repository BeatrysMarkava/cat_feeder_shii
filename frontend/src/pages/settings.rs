use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{FileReader, HtmlInputElement};

use crate::app::AppState;
use crate::tauri_api;

#[component]
pub fn SettingsPage<F>(
    app_state: ReadSignal<AppState>,
    set_app_state: WriteSignal<AppState>,
    on_feeder_list: F,
) -> impl IntoView
where
    F: Fn() + Copy + 'static,
{
    let input_ref = NodeRef::<html::Input>::new();

    let open_picker = move |_| {
        tauri_api::report_button_click("settings_change_photo_clicked", None);
        if let Some(input) = input_ref.get() {
            input.click();
        }
    };

    let change_photo = move |ev: ev::Event| {
        let Some(target) = ev.target() else {
            return;
        };

        let Ok(input) = target.dyn_into::<HtmlInputElement>() else {
            return;
        };

        let Some(files) = input.files() else {
            return;
        };

        let Some(file) = files.get(0) else {
            return;
        };

        let Ok(reader) = FileReader::new() else {
            return;
        };

        let reader_clone = reader.clone();
        let set_app_state = set_app_state;
        let onload = Closure::wrap(Box::new(move || {
            let Ok(result) = reader_clone.result() else {
                return;
            };

            if let Some(image_url) = result.as_string() {
                set_app_state.update(|state| {
                    state.cat_photo = image_url;
                });
            }
        }) as Box<dyn FnMut()>);

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));

        if reader.read_as_data_url(&file).is_ok() {
            onload.forget();
        }
    };

    let update_name = move |ev| {
        let value = event_target_value(&ev);
        set_app_state.update(|state| state.cat_name = value);
    };

    let toggle_notifications = move |_| {
        tauri_api::report_button_click("settings_notifications_clicked", None);
        set_app_state.update(|state| {
            state.notifications_enabled = !state.notifications_enabled;
        });
    };

    let toggle_connection = move |_| {
        tauri_api::report_button_click("settings_connection_clicked", None);
        set_app_state.update(|state| {
            state.feeder_connected = !state.feeder_connected;
        });
    };

    let refill_hopper = move |_| {
        tauri_api::report_button_click("settings_refill_hopper_clicked", None);
        set_app_state.update(|state| state.refill_hopper());
    };

    let return_to_feeders = move |_| {
        tauri_api::report_button_click("settings_switch_feeder_clicked", None);
        set_app_state.update(|state| state.clear_selected_feeder());
        on_feeder_list();
    };

    view! {
        <section class="page">
            <div class="settings-hero">
                <div class="settings-photo-frame">
                    <img
                        src=move || app_state.get().cat_photo
                        alt="Cat photo"
                        class="settings-photo"
                    />
                </div>
            </div>

            <div class="settings-panel">
                <label class="settings-field">
                    <span class="settings-label">"Cat name"</span>
                    <input
                        class="settings-input"
                        type="text"
                        prop:value=move || app_state.get().cat_name
                        on:input=update_name
                    />
                </label>

                <button class="settings-action" on:click=open_picker>
                    <div>
                        <span class="settings-label">"Change profile photo"</span>
                        <span class="settings-hint">"Upload a new picture for the home screen."</span>
                    </div>
                    <span class="settings-arrow">">"</span>
                </button>

                <button class="settings-action" on:click=toggle_notifications>
                    <div>
                        <span class="settings-label">"Notifications"</span>
                        <span class="settings-hint">"Get alerts about feeding and refill events."</span>
                    </div>
                    <span class=move || {
                        if app_state.get().notifications_enabled {
                            "settings-tag enabled"
                        } else {
                            "settings-tag"
                        }
                    }>
                        {move || {
                            if app_state.get().notifications_enabled {
                                "On"
                            } else {
                                "Off"
                            }
                        }}
                    </span>
                </button>

                <button class="settings-action" on:click=toggle_connection>
                    <div>
                        <span class="settings-label">"Feeder connection"</span>
                        <span class="settings-hint">"Simulate online or offline device state."</span>
                    </div>
                    <span class=move || {
                        if app_state.get().feeder_connected {
                            "settings-tag enabled"
                        } else {
                            "settings-tag warning"
                        }
                    }>
                        {move || {
                            if app_state.get().feeder_connected {
                                "Online"
                            } else {
                                "Offline"
                            }
                        }}
                    </span>
                </button>

                <button class="settings-action" on:click=refill_hopper>
                    <div>
                        <span class="settings-label">"Refill hopper"</span>
                        <span class="settings-hint">
                            {move || format!("Current stock: {}%", app_state.get().hopper_level)}
                        </span>
                    </div>
                    <span class="settings-tag enabled">"Fill"</span>
                </button>

                <button class="settings-action" on:click=return_to_feeders>
                    <div>
                        <span class="settings-label">"Switch feeder"</span>
                        <span class="settings-hint">
                            "Return to the feeder selection screen."
                        </span>
                    </div>
                    <span class="settings-arrow">">"</span>
                </button>
            </div>

            <input
                node_ref=input_ref
                class="hidden-input"
                type="file"
                accept="image/*"
                on:change=change_photo
            />
        </section>
    }
}
