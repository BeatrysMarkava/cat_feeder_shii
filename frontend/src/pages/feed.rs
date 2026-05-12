use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::app::{AppState, portion_text};
use crate::tauri_api;

const FEED_STATUS_CHECK_ATTEMPTS: usize = 3;
const FEED_STATUS_CHECK_INTERVAL_MS: i32 = 5_000;

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
        tauri_api::report_button_click("feed_portion_decrease_clicked", None);
        if portion.get() > 1 {
            set_portion.update(|value| *value -= 1);
        }
    };

    let increase = move |_| {
        tauri_api::report_button_click("feed_portion_increase_clicked", None);
        if portion.get() < 5 {
            set_portion.update(|value| *value += 1);
        }
    };

    let dispense = move |_| {
        tauri_api::report_button_click(
            "feed_dispense_clicked",
            Some(format!("portion={}", portion.get())),
        );
        if is_sending.get() {
            return;
        }

        let selected = portion.get();
        set_is_sending.set(true);
        set_delivery_status.set(String::from("Sending feeding request."));

        spawn_local(async move {
            let _ = tauri_api::frontend_action(
                "feed_now_request_started",
                Some(format!("portion={selected}")),
            )
            .await;

            match api::create_feed_now_command(selected).await {
                Ok(command) => {
                    set_delivery_status.set(String::from("Waiting for feeder confirmation."));
                    let _ = tauri_api::frontend_action(
                        "feed_now_command_queued",
                        Some(format!(
                            "command_id={}, status={}, retry={}/{}",
                            command.id, command.status, command.retry_count, command.max_attempts
                        )),
                    )
                    .await;
                    let _ = api::track_client_action(
                        "feed_now_queued",
                        Some(format!("command_id={}, portion={selected}", command.id)),
                    )
                    .await;

                    let mut finished = false;
                    for attempt in 1..=FEED_STATUS_CHECK_ATTEMPTS {
                        sleep_ms(FEED_STATUS_CHECK_INTERVAL_MS).await;

                        match api::fetch_command(command.id).await {
                            Ok(updated) => match updated.status.as_str() {
                                "completed" => {
                                    let _ = tauri_api::frontend_action(
                                        "feed_now_status_completed",
                                        Some(format!(
                                            "command_id={}, attempt={}, retry={}/{}",
                                            updated.id,
                                            attempt,
                                            updated.retry_count,
                                            updated.max_attempts
                                        )),
                                    )
                                    .await;
                                    set_app_state.update(|state| state.feed_now(selected));
                                    set_delivery_status.set(String::from("Food is ready."));
                                    set_was_dispensed.set(true);
                                    let _ = api::track_client_action(
                                        "feed_now_completed",
                                        Some(format!(
                                            "command_id={}, portion={selected}",
                                            updated.id
                                        )),
                                    )
                                    .await;
                                    sleep_ms(1_200).await;
                                    on_back();
                                    finished = true;
                                    break;
                                }
                                "failed" => {
                                    let message = updated.message.unwrap_or_else(|| {
                                        String::from("The feeder did not confirm delivery")
                                    });
                                    set_delivery_status
                                        .set(String::from("Feeder did not confirm feeding."));
                                    let _ = tauri_api::frontend_action(
                                        "feed_now_status_failed",
                                        Some(format!(
                                            "command_id={}, attempt={}, retry={}/{}, message={}",
                                            updated.id,
                                            attempt,
                                            updated.retry_count,
                                            updated.max_attempts,
                                            message
                                        )),
                                    )
                                    .await;
                                    let _ = api::track_client_action(
                                        "feed_now_failed",
                                        Some(format!(
                                            "command_id={}, retry={}/{}, message={}",
                                            updated.id,
                                            updated.retry_count,
                                            updated.max_attempts,
                                            message
                                        )),
                                    )
                                    .await;
                                    finished = true;
                                    break;
                                }
                                _ => {
                                    let _ = tauri_api::frontend_action(
                                        "feed_now_status_waiting",
                                        Some(format!(
                                            "command_id={}, attempt={}, status={}, retry={}/{}",
                                            updated.id,
                                            attempt,
                                            updated.status,
                                            updated.retry_count,
                                            updated.max_attempts
                                        )),
                                    )
                                    .await;
                                }
                            },
                            Err(error) => {
                                let _ = tauri_api::frontend_action(
                                    "feed_now_status_check_error",
                                    Some(format!(
                                        "command_id={}, attempt={}, error={}",
                                        command.id, attempt, error
                                    )),
                                )
                                .await;
                            }
                        }
                    }

                    if !finished {
                        set_delivery_status.set(String::from(
                            "Feeder did not confirm feeding. Check the device.",
                        ));
                        let _ = tauri_api::frontend_action(
                            "feed_now_wait_timeout",
                            Some(format!(
                                "command_id={}, attempts={}, interval_ms={}",
                                command.id,
                                FEED_STATUS_CHECK_ATTEMPTS,
                                FEED_STATUS_CHECK_INTERVAL_MS
                            )),
                        )
                        .await;
                        let _ = api::track_client_action(
                            "feed_now_wait_timeout",
                            Some(format!("command_id={}", command.id)),
                        )
                        .await;
                    }
                }
                Err(error) => {
                    let detail = error.clone();
                    set_delivery_status.set(String::from(
                        "Could not send the feeding request. Check the connection.",
                    ));
                    let _ = tauri_api::frontend_action(
                        "feed_now_connection_error",
                        Some(format!("portion={selected}, error={detail}")),
                    )
                    .await;

                    match tauri_api::feed_now_via_tauri(selected).await {
                        Ok(message) => {
                            let _ = tauri_api::frontend_action(
                                "feed_now_direct_fallback_result",
                                Some(message),
                            )
                            .await;
                            let _ = api::track_client_action(
                                "feed_now_direct_fallback",
                                Some(format!("portion={selected}, connection_error={detail}")),
                            )
                            .await;
                        }
                        Err(direct_error) => {
                            let _ = tauri_api::frontend_action(
                                "feed_now_direct_fallback_error",
                                Some(format!("connection={detail}, direct={direct_error}")),
                            )
                            .await;
                            let _ = api::track_client_action(
                                "feed_now_error",
                                Some(format!("connection={detail}, direct={direct_error}")),
                            )
                            .await;
                        }
                    }
                }
            }

            set_is_sending.set(false);
        });
    };

    let feed_again = move |_| {
        tauri_api::report_button_click("feed_again_clicked", None);
        set_was_dispensed.set(false);
        set_delivery_status.set(String::new());
    };

    view! {
        <section class="page">
            <div class="top-bar">
                <button
                    class="back-button"
                    on:click=move |_| {
                        tauri_api::report_button_click("feed_back_clicked", None);
                        on_back();
                    }
                >
                    "<"
                </button>
                <div class="app-title">"Feed Now"</div>
            </div>

            {move || {
                if was_dispensed.get() {
                    view! {
                        <div class="success-panel">
                            <p class="eyebrow">"Feeding complete"</p>
                            <h2 class="success-title">
                                {move || format!("{} was fed", app_state.get().cat_name)}
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
                                <button
                                    class="cta-button cta-primary"
                                    on:click=move |_| {
                                        tauri_api::report_button_click("feed_back_home_clicked", None);
                                        on_back();
                                    }
                                >
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
                                        String::from("Waiting for feeder...")
                                    } else {
                                        format!("Dispense {}", portion_text(portion.get()))
                                    }
                                }}
                            </button>

                            <Show when=move || !delivery_status.get().is_empty()>
                                <p class="panel-subtitle">{move || delivery_status.get()}</p>
                            </Show>
                        </div>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}
