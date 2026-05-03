use leptos::prelude::*;

use crate::app::{AppState, format_schedule_label, portion_text};

#[component]
pub fn HomePage<F1, F2>(
    app_state: ReadSignal<AppState>,
    on_feed_now: F1,
    on_open_schedule: F2,
) -> impl IntoView
where
    F1: Fn() + Copy + 'static,
    F2: Fn() + Copy + 'static,
{
    view! {
        <section class="page page-home">
            <div class="hero-card">
                <div class="home-avatar">
                    <img
                        src=move || app_state.get().cat_photo
                        alt="Cat photo"
                        class="home-avatar-image"
                    />
                </div>
            </div>

            <div class="status-strip">
                <div class="status-chip">
                    <span class="chip-label">"Feeder"</span>
                    <span class=move || {
                        if app_state.get().feeder_connected {
                            "chip-value online"
                        } else {
                            "chip-value offline"
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
                </div>

                <div class="status-chip">
                    <span class="chip-label">"Food left"</span>
                    <span class="chip-value warm">
                        {move || format!("{}%", app_state.get().hopper_level)}
                    </span>
                </div>

                <div class="status-chip">
                    <span class="chip-label">"Schedule sync"</span>
                    <span class=move || {
                        if app_state.get().schedule_error.is_some() {
                            "chip-value offline"
                        } else if app_state.get().schedule_loading {
                            "chip-value muted"
                        } else {
                            "chip-value online"
                        }
                    }>
                        {move || {
                            if app_state.get().schedule_error.is_some() {
                                "Error"
                            } else if app_state.get().schedule_loading {
                                "Loading"
                            } else {
                                "Ready"
                            }
                        }}
                    </span>
                </div>
            </div>

            <div class="metrics-grid">
                <div class="metric-card">
                    <span class="metric-label">"Last feeding"</span>
                    <strong class="metric-value">
                        {move || app_state.get().last_fed_label}
                    </strong>
                </div>

                <div class="metric-card">
                    <span class="metric-label">"Next automatic meal"</span>
                    <strong class="metric-value">
                        {move || app_state.get().next_feeding_label()}
                    </strong>
                </div>
            </div>

            <div class="cta-grid">
                <button class="cta-button cta-primary" on:click=move |_| on_feed_now()>
                    <span class="cta-title">"Feed Now"</span>
                </button>

                <button class="cta-button cta-secondary" on:click=move |_| on_open_schedule()>
                    <span class="cta-title">"Schedule Feeding"</span>
                </button>
            </div>

            <div class="panel">
                <div class="panel-header">
                    <div>
                        <p class="panel-title">"Upcoming schedule"</p>
                        <p class="panel-subtitle">
                            {move || {
                                if app_state.get().schedule_loading {
                                    String::from("Loading schedules from the server...")
                                } else if let Some(error) = app_state.get().schedule_error {
                                    format!("Backend problem: {error}")
                                } else {
                                    format!(
                                        "{} active meals - {} total portions",
                                        app_state.get().active_schedule_count(),
                                        app_state.get().daily_portions()
                                    )
                                }
                            }}
                        </p>
                    </div>
                    <button class="text-button" on:click=move |_| on_open_schedule()>
                        "Open"
                    </button>
                </div>

                <Show
                    when=move || !app_state.get().schedule.is_empty()
                    fallback=move || {
                        view! {
                            <div class="empty-state">
                                <p class="empty-title">"No meals saved yet"</p>
                            </div>
                        }
                    }
                >
                    <div class="schedule-preview">
                        <For
                            each=move || {
                                app_state
                                    .get()
                                    .schedule
                                    .into_iter()
                                    .take(3)
                                    .collect::<Vec<_>>()
                            }
                            key=|entry| entry.id
                            children=move |entry| {
                                view! {
                                    <div class="schedule-row">
                                        <div>
                                            <div class="schedule-label">
                                                {format_schedule_label(&entry)}
                                            </div>
                                            <div class="schedule-detail">
                                                {portion_text(entry.portion)}
                                            </div>
                                        </div>
                                        <span class=if entry.enabled {
                                            "pill-badge active"
                                        } else {
                                            "pill-badge inactive"
                                        }>
                                            {if entry.enabled { "Active" } else { "Paused" }}
                                        </span>
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
