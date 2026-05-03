use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::app::{AppState, EventTone, portion_text};

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
pub fn FeedNowPage<F>(
    app_state: ReadSignal<AppState>,
    set_app_state: WriteSignal<AppState>,
    on_back: F,
) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    let (portion, set_portion) = signal(app_state.get().preferred_portion.clamp(1, 5));
    let (was_dispensed, set_was_dispensed) = signal(false);
    let (is_sending, set_is_sending) = signal(false);
    let (delivery_status, set_delivery_status) = signal(String::new());

    let decrease = move |_| {
        if portion.get() > 1 {
            set_portion.update(|value| *value -= 1);
        }
    };

    let increase = move |_| {
        if portion.get() < 5 {
            set_portion.update(|value| *value += 1);
        }
    };

    let dispense = move |_| {
        if is_sending.get() {
            return;
        }

        let selected = portion.get();
        set_is_sending.set(true);
        set_delivery_status.set(String::from("Sending command to server..."));

        spawn_local(async move {
            match api::create_feed_now_command(selected).await {
                Ok(command) => {
                    set_app_state.update(|state| {
                        state.push_event(
                            String::from("Feed command sent to ESP32-C6"),
                            format!("Command #{} queued", command.id),
                            String::from("Just now"),
                            EventTone::Info,
                        );
                    });
                    set_delivery_status.set(format!(
                        "Command #{} queued. Waiting for delivery confirmation.",
                        command.id
                    ));
                    set_was_dispensed.set(true);

                    let mut finished = false;
                    for _ in 0..40 {
                        sleep_ms(3_000).await;

                        match api::fetch_command(command.id).await {
                            Ok(updated) => match updated.status.as_str() {
                                "completed" => {
                                    set_app_state.update(|state| state.feed_now(selected));
                                    set_delivery_status.set(String::from("Food is ready."));
                                    finished = true;
                                    break;
                                }
                                "failed" => {
                                    set_app_state.update(|state| {
                                        state.push_event(
                                            String::from("Feed command failed"),
                                            updated.message.unwrap_or_else(|| {
                                                String::from("ESP32-C6 did not confirm delivery")
                                            }),
                                            String::from("Just now"),
                                            EventTone::Warning,
                                        );
                                    });
                                    set_delivery_status.set(format!(
                                        "Delivery failed after {}/{} attempts.",
                                        updated.retry_count, updated.max_attempts
                                    ));
                                    finished = true;
                                    break;
                                }
                                "claimed" => {
                                    set_delivery_status.set(format!(
                                        "ESP32-C6 claimed command #{}. Attempt {}/{}.",
                                        updated.id,
                                        updated.retry_count + 1,
                                        updated.max_attempts
                                    ));
                                }
                                _ => {
                                    set_delivery_status.set(format!(
                                        "Command #{} is queued. Retry {}/{}.",
                                        updated.id, updated.retry_count, updated.max_attempts
                                    ));
                                }
                            },
                            Err(error) => {
                                set_delivery_status
                                    .set(format!("Could not check command status: {error}"));
                            }
                        }
                    }

                    if !finished {
                        set_delivery_status.set(String::from(
                            "Still waiting for ESP32-C6 confirmation. Check command status again later.",
                        ));
                    }
                }
                Err(error) => {
                    set_app_state.update(|state| {
                        state.push_event(
                            String::from("Feed command failed"),
                            error,
                            String::from("Just now"),
                            EventTone::Warning,
                        );
                    });
                }
            }

            set_is_sending.set(false);
        });
    };

    let feed_again = move |_| {
        set_was_dispensed.set(false);
    };

    view! {
        <section class="page">
            <div class="top-bar">
                <button class="back-button" on:click=move |_| on_back()>
                    "<"
                </button>
                <div class="app-title">"Feed Now"</div>
            </div>

            {move || {
                if was_dispensed.get() {
                    view! {
                        <div class="success-panel">
                            <p class="eyebrow">"Command queued"</p>
                            <h2 class="success-title">
                                {move || format!("ESP32-C6 will dispense {}", portion_text(portion.get()))}
                            </h2>
                            <p class="success-copy">
                                {move || {
                                    let status = delivery_status.get();
                                    if status.is_empty() {
                                        String::from("Waiting for command status.")
                                    } else {
                                        status
                                    }
                                }}
                            </p>

                            <div class="success-actions">
                                <button class="cta-button cta-secondary" on:click=feed_again>
                                    <span class="cta-title">"Feed Another Portion"</span>
                                    <span class="cta-copy">"Adjust the amount and dispense again."</span>
                                </button>
                                <button class="cta-button cta-primary" on:click=move |_| on_back()>
                                    <span class="cta-title">"Back Home"</span>
                                    <span class="cta-copy">"Return to feeder overview."</span>
                                </button>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="panel panel-tight">
                            <div class="panel-header">
                                <div>
                                    <p class="panel-title">"Manual feeding"</p>
                                    <p class="panel-subtitle">
                                        {move || {
                                            if app_state.get().feeder_connected {
                                                String::from("Feeder is online and ready.")
                                            } else {
                                                String::from("Feeder is offline. Dispensing is simulated.")
                                            }
                                        }}
                                    </p>
                                </div>
                                <span class="pill-badge active">
                                    {move || format!("{}% full", app_state.get().hopper_level)}
                                </span>
                            </div>

                            <div class="feed-controls">
                                <button class="portion-btn" on:click=decrease>
                                    "-"
                                </button>
                                <div class="portion-value">{move || portion.get()}</div>
                                <button class="portion-btn" on:click=increase>
                                    "+"
                                </button>
                            </div>

                            <div class="portion-helper">
                                {move || portion_text(portion.get())}
                            </div>

                            <div class="pills-row">
                                <img
                                    class=move || format!("pills-size-{}", portion.get().clamp(1, 3))
                                    src="assets/pills_feed.png"
                                    alt="Food portions"
                                />
                            </div>

                            <button
                                class="feed-now-button"
                                on:click=dispense
                                disabled=move || is_sending.get()
                            >
                                {move || {
                                    if is_sending.get() {
                                        String::from("Sending command...")
                                    } else {
                                        format!("Dispense {}", portion_text(portion.get()))
                                    }
                                }}
                            </button>
                        </div>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}
