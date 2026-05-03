use leptos::prelude::*;

use crate::app::AppState;

#[component]
pub fn NotificationsPage(app_state: ReadSignal<AppState>) -> impl IntoView {
    view! {
        <section class="page">
            <div class="panel hero-panel">
                <div class="illustration-wrap">
                    <img src="assets/notification_big.png" alt="Notifications" />
                </div>
                <p class="panel-title">"Activity feed"</p>
                <p class="panel-subtitle">
                    {move || {
                        if app_state.get().notifications_enabled {
                            String::from("Push notifications are enabled for feeder events.")
                        } else {
                            String::from("Push notifications are off, but event history is still saved here.")
                        }
                    }}
                </p>
            </div>

            <div class="timeline">
                <div class="timeline-list">
                    <For
                        each=move || app_state.get().activity
                        key=|item| item.id
                        children=move |item| {
                            view! {
                                <div class="timeline-item">
                                    <div class="timeline-side">
                                        <div class=format!("timeline-dot {}", item.tone.class_name())></div>
                                        <div class="timeline-line"></div>
                                    </div>
                                    <div class="timeline-card">
                                        <div class="timeline-card-header">
                                            <strong>{item.title}</strong>
                                            <span>{item.time}</span>
                                        </div>
                                        <p>{item.detail}</p>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </section>
    }
}
