use common::{CreateScheduleRequest, UpdateScheduleRequest};
use leptos::{ev, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::app::{AppState, portion_text, sync_schedules};
use crate::tauri_api;

#[component]
pub fn SchedulePage<F>(
    app_state: ReadSignal<AppState>,
    set_app_state: WriteSignal<AppState>,
    on_back: F,
) -> impl IntoView
where
    F: Fn() + Copy + 'static,
{
    let (selected_date, set_selected_date) = signal(String::new());
    let (selected_time, set_selected_time) = signal(String::new());
    let (portion, set_portion) = signal(1_u8);
    let (is_saving, set_is_saving) = signal(false);
    let (status_message, set_status_message) = signal(String::new());

    let update_date = move |ev: ev::Event| {
        set_selected_date.set(event_target_value(&ev));
    };

    let update_time = move |ev: ev::Event| {
        set_selected_time.set(event_target_value(&ev));
    };

    let can_save_schedule =
        move || !selected_date.get().is_empty() && !selected_time.get().is_empty();

    let decrease = move |_| {
        tauri_api::report_button_click("schedule_portion_decrease_clicked", None);
        if portion.get() > 1 {
            set_portion.update(|value| *value -= 1);
        }
    };

    let increase = move |_| {
        tauri_api::report_button_click("schedule_portion_increase_clicked", None);
        if portion.get() < 5 {
            set_portion.update(|value| *value += 1);
        }
    };

    let create_schedule = move |_| {
        tauri_api::report_button_click(
            "schedule_save_clicked",
            Some(format!(
                "date={}, time={}, portion={}",
                selected_date.get(),
                selected_time.get(),
                portion.get()
            )),
        );
        if !can_save_schedule() || is_saving.get() {
            return;
        }

        let date = selected_date.get();
        let time = selected_time.get();
        let selected_portion = portion.get();

        set_is_saving.set(true);
        set_status_message.set(String::new());

        spawn_local(async move {
            let payload = CreateScheduleRequest {
                feeding_date: Some(date),
                feeding_day: String::from("Monday"),
                feeding_time: time,
                timezone: Some(String::from("Europe/Bratislava")),
                portion: selected_portion,
            };

            match api::create_schedule(&payload).await {
                Ok(schedule) => {
                    set_app_state.update(|state| {
                        state.upsert_schedule(schedule);
                    });
                    set_selected_date.set(String::new());
                    set_selected_time.set(String::new());
                    set_portion.set(1);
                    set_status_message.set(String::from("Feeding schedule saved."));
                }
                Err(error) => {
                    set_app_state.update(|state| {
                        state.set_schedule_error(error.clone());
                    });
                    set_status_message.set(format!("Could not save feeding: {error}"));
                }
            }

            set_is_saving.set(false);
        });
    };

    let refresh_schedule = move |_| {
        tauri_api::report_button_click("schedule_refresh_clicked", None);
        sync_schedules(set_app_state);
    };

    view! {
        <section class="page">
            <div class="top-bar">
                <button
                    class="back-button"
                    on:click=move |_| {
                        tauri_api::report_button_click("schedule_back_clicked", None);
                        on_back();
                    }
                >
                    "<"
                </button>
                <div class="app-title">"Schedule Feeding"</div>
            </div>

            <div class="panel panel-tight">
                <div class="panel-header">
                    <div>
                        <p class="panel-title">"Create feeding slot"</p>
                        <p class="panel-subtitle">
                            "Choose a date and time first, then choose the portion size."
                        </p>
                    </div>
                </div>

                <div class="schedule-form">
                    <label class="settings-field">
                        <span class="settings-label">"Date"</span>
                        <input
                            class="settings-input"
                            type="date"
                            prop:value=move || selected_date.get()
                            on:input=update_date
                        />
                    </label>

                    <label class="settings-field">
                        <span class="settings-label">"Time"</span>
                        <input
                            class="settings-input"
                            type="time"
                            prop:value=move || selected_time.get()
                            on:input=update_time
                        />
                    </label>
                </div>

                <Show
                    when=can_save_schedule
                    fallback=move || {
                        view! {
                            <div class="empty-state">
                                <p class="empty-title">"Pick date and time first"</p>
                                <p class="empty-copy">
                                    "The portion selector unlocks after both fields are filled."
                                </p>
                            </div>
                        }
                    }
                >
                    <div class="portion-picker">
                        <div class="panel-header">
                            <div>
                                <p class="panel-title">"Portion size"</p>
                                <p class="panel-subtitle">
                                    {move || {
                                        format!(
                                            "Meal time: {} at {}",
                                            selected_date.get(),
                                            selected_time.get()
                                        )
                                    }}
                                </p>
                            </div>
                            <span class="pill-badge active">{move || portion_text(portion.get())}</span>
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
                            "The feeder will use this time for the next scheduled meal."
                        </div>

                        <button
                            class="feed-now-button"
                            on:click=create_schedule
                            disabled=move || is_saving.get()
                        >
                            {move || {
                                if is_saving.get() {
                                    String::from("Saving...")
                                } else {
                                    String::from("Save feeding schedule")
                                }
                            }}
                        </button>
                    </div>
                </Show>

                <Show when=move || !status_message.get().is_empty()>
                    <p class="inline-status">{move || status_message.get()}</p>
                </Show>
            </div>

            <div class="panel panel-tight schedule-list-panel">
                <div class="panel-header">
                    <div>
                        <p class="panel-title">"Saved feedings"</p>
                        <p class="panel-subtitle">
                            {move || {
                                if app_state.get().schedule_loading {
                                    String::from("Loading saved meals...")
                                } else if let Some(error) = app_state.get().schedule_error {
                                    format!("Could not load meals: {error}")
                                } else {
                                    format!(
                                        "{} saved meals",
                                        app_state.get().schedule.len()
                                    )
                                }
                            }}
                        </p>
                    </div>
                    <button class="text-button" on:click=refresh_schedule>
                        "Refresh"
                    </button>
                </div>

                <Show
                    when=move || !app_state.get().schedule.is_empty()
                    fallback=move || {
                        view! {
                            <div class="empty-state">
                                <p class="empty-title">"No saved meals yet"</p>
                                <p class="empty-copy">
                                    "Create the first scheduled meal above."
                                </p>
                            </div>
                        }
                    }
                >
                    <div class="schedule-editor">
                        <For
                            each=move || app_state.get().schedule
                            key=|entry| entry.id
                            children=move |entry| {
                                let toggle_id = entry.id;
                                let delete_id = entry.id;
                                let new_enabled = !entry.enabled;
                                let schedule_day_label = entry.feeding_day.clone();
                                let schedule_detail = format!(
                                    "{} - {} - next {}",
                                    entry.feeding_time,
                                    portion_text(entry.portion),
                                    entry.next_run_at
                                );

                                let toggle = move |_| {
                                    tauri_api::report_button_click(
                                        "schedule_toggle_clicked",
                                        Some(format!("id={toggle_id}, enabled={new_enabled}")),
                                    );
                                    spawn_local(async move {
                                        let payload = UpdateScheduleRequest {
                                            feeding_date: None,
                                            feeding_day: None,
                                            feeding_time: None,
                                            timezone: None,
                                            portion: None,
                                            enabled: Some(new_enabled),
                                        };

                                        match api::update_schedule(toggle_id, &payload).await {
                                            Ok(updated) => {
                                                set_app_state.update(|state| {
                                                    state.upsert_schedule(updated);
                                                });
                                            }
                                            Err(error) => {
                                                set_app_state.update(|state| {
                                                    state.set_schedule_error(error.clone());
                                                });
                                            }
                                        }
                                    });
                                };

                                let delete = move |_| {
                                    tauri_api::report_button_click(
                                        "schedule_delete_clicked",
                                        Some(format!("id={delete_id}")),
                                    );
                                    spawn_local(async move {
                                        match api::delete_schedule(delete_id).await {
                                            Ok(()) => {
                                                set_app_state.update(|state| {
                                                    state.remove_schedule(delete_id);
                                                });
                                            }
                                            Err(error) => {
                                                set_app_state.update(|state| {
                                                    state.set_schedule_error(error.clone());
                                                });
                                            }
                                        }
                                    });
                                };

                                view! {
                                    <div class="schedule-card">
                                        <div class="schedule-card-head">
                                            <div>
                                                <div class="schedule-label">{schedule_day_label}</div>
                                                <div class="schedule-detail">{schedule_detail}</div>
                                            </div>
                                            <span class=if entry.enabled {
                                                "pill-badge active"
                                            } else {
                                                "pill-badge inactive"
                                            }>
                                                {if entry.enabled { "Active" } else { "Paused" }}
                                            </span>
                                        </div>

                                        <div class="schedule-actions">
                                            <button class="secondary-inline-button" on:click=toggle>
                                                {if entry.enabled { "Pause" } else { "Activate" }}
                                            </button>
                                            <button class="secondary-inline-button danger" on:click=delete>
                                                "Delete"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </Show>
            </div>
        </section>
    }
}
