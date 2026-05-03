use common::FeedingSchedule;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::bottom_navigation::BottomNavigation;
use crate::pages::{
    calendar::SchedulePage, feed::FeedNowPage, home::HomePage, notifications::NotificationsPage,
    settings::SettingsPage,
};
use crate::styles::Styles;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    FeedNow,
    Schedule,
    Notifications,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventTone {
    Success,
    Warning,
    Info,
}

impl EventTone {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ActivityEvent {
    pub id: u32,
    pub title: String,
    pub detail: String,
    pub time: String,
    pub tone: EventTone,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AppState {
    pub cat_name: String,
    pub cat_photo: String,
    pub feeder_connected: bool,
    pub notifications_enabled: bool,
    pub hopper_level: u8,
    pub last_fed_label: String,
    pub preferred_portion: u8,
    pub schedule: Vec<FeedingSchedule>,
    pub schedule_loading: bool,
    pub schedule_error: Option<String>,
    pub activity: Vec<ActivityEvent>,
    next_event_id: u32,
}

impl AppState {
    pub fn demo() -> Self {
        Self {
            cat_name: String::from("Barsik"),
            cat_photo: String::from("assets/image.png"),
            feeder_connected: true,
            notifications_enabled: true,
            hopper_level: 78,
            last_fed_label: String::from("Today at 12:30 - 2 portions"),
            preferred_portion: 2,
            schedule: Vec::new(),
            schedule_loading: true,
            schedule_error: None,
            activity: vec![
                ActivityEvent {
                    id: 1,
                    title: String::from("Feeder is online"),
                    detail: String::from("Wi-Fi connection is stable"),
                    time: String::from("Now"),
                    tone: EventTone::Info,
                },
                ActivityEvent {
                    id: 2,
                    title: String::from("Barsik was fed"),
                    detail: String::from("Automatic schedule - 2 portions"),
                    time: String::from("Today at 12:30"),
                    tone: EventTone::Success,
                },
                ActivityEvent {
                    id: 3,
                    title: String::from("Schedule storage moved to the server"),
                    detail: String::from(
                        "Create meals in Schedule Feeding to save them in the database",
                    ),
                    time: String::from("Now"),
                    tone: EventTone::Info,
                },
            ],
            next_event_id: 4,
        }
    }

    pub fn next_feeding_label(&self) -> String {
        self.schedule
            .iter()
            .find(|entry| entry.enabled)
            .map(format_schedule_label)
            .unwrap_or_else(|| String::from("No active schedule"))
    }

    pub fn active_schedule_count(&self) -> usize {
        self.schedule.iter().filter(|entry| entry.enabled).count()
    }

    pub fn daily_portions(&self) -> u32 {
        self.schedule
            .iter()
            .filter(|entry| entry.enabled)
            .map(|entry| u32::from(entry.portion))
            .sum()
    }

    pub fn feed_now(&mut self, portions: u8) {
        let portion_text = portion_text(portions);
        self.preferred_portion = portions;
        self.last_fed_label = format!("Just now - {}", portion_text);
        self.hopper_level = self.hopper_level.saturating_sub(portions.saturating_mul(8));

        self.push_event(
            format!("{} was fed", self.cat_name),
            format!("Manual feeding - {}", portion_text),
            String::from("Just now"),
            EventTone::Success,
        );

        if self.hopper_level <= 25 {
            self.push_event(
                String::from("Time to refill the feeder"),
                format!("Only {}% of food remains", self.hopper_level),
                String::from("Just now"),
                EventTone::Warning,
            );
        }
    }

    pub fn push_event(&mut self, title: String, detail: String, time: String, tone: EventTone) {
        let event = ActivityEvent {
            id: self.next_event_id,
            title,
            detail,
            time,
            tone,
        };
        self.next_event_id += 1;
        self.activity.insert(0, event);
    }

    pub fn refill_hopper(&mut self) {
        self.hopper_level = 100;
        self.push_event(
            String::from("Hopper refilled"),
            String::from("Food stock is back to 100%"),
            String::from("Just now"),
            EventTone::Info,
        );
    }

    pub fn set_schedules(&mut self, mut schedules: Vec<FeedingSchedule>) {
        schedules.sort_by(compare_schedules);
        self.schedule = schedules;
        self.schedule_loading = false;
        self.schedule_error = None;
    }

    pub fn set_schedule_error(&mut self, message: String) {
        self.schedule_loading = false;
        self.schedule_error = Some(message);
    }

    pub fn upsert_schedule(&mut self, schedule: FeedingSchedule) {
        if let Some(existing) = self
            .schedule
            .iter_mut()
            .find(|entry| entry.id == schedule.id)
        {
            *existing = schedule;
        } else {
            self.schedule.push(schedule);
        }

        self.schedule.sort_by(compare_schedules);
        self.schedule_error = None;
        self.schedule_loading = false;
    }

    pub fn remove_schedule(&mut self, schedule_id: i64) {
        self.schedule.retain(|entry| entry.id != schedule_id);
        self.schedule_error = None;
    }
}

pub fn portion_text(portions: u8) -> String {
    if portions == 1 {
        String::from("1 portion")
    } else {
        format!("{portions} portions")
    }
}

pub fn day_options() -> [&'static str; 7] {
    [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ]
}

pub fn format_schedule_label(entry: &FeedingSchedule) -> String {
    if let Some(date) = &entry.feeding_date {
        return format!("{date} at {} ({})", entry.feeding_time, entry.timezone);
    }

    format!(
        "{} at {} ({})",
        entry.feeding_day, entry.feeding_time, entry.timezone
    )
}

fn day_order(day: &str) -> usize {
    day_options()
        .iter()
        .position(|candidate| *candidate == day)
        .unwrap_or(usize::MAX)
}

fn compare_schedules(left: &FeedingSchedule, right: &FeedingSchedule) -> std::cmp::Ordering {
    if left.feeding_date.is_some() || right.feeding_date.is_some() {
        return left
            .feeding_date
            .cmp(&right.feeding_date)
            .then_with(|| left.feeding_time.cmp(&right.feeding_time))
            .then_with(|| left.id.cmp(&right.id));
    }

    day_order(&left.feeding_day)
        .cmp(&day_order(&right.feeding_day))
        .then_with(|| left.feeding_time.cmp(&right.feeding_time))
        .then_with(|| left.id.cmp(&right.id))
}

pub fn sync_schedules(set_app_state: WriteSignal<AppState>) {
    set_app_state.update(|state| {
        state.schedule_loading = true;
        state.schedule_error = None;
    });

    spawn_local(async move {
        match api::fetch_schedules().await {
            Ok(schedules) => {
                set_app_state.update(|state| state.set_schedules(schedules));
            }
            Err(error) => {
                set_app_state.update(|state| {
                    state.set_schedule_error(error.clone());
                    state.push_event(
                        String::from("Schedule sync failed"),
                        error,
                        String::from("Now"),
                        EventTone::Warning,
                    );
                });
            }
        }
    });
}

#[component]
pub fn App() -> impl IntoView {
    let (page, set_page) = signal(Page::Home);
    let (app_state, set_app_state) = signal(AppState::demo());
    let (did_bootstrap, set_did_bootstrap) = signal(false);

    Effect::new(move |_| {
        if !did_bootstrap.get() {
            set_did_bootstrap.set(true);
            sync_schedules(set_app_state);
        }
    });

    view! {
        <style>{Styles::GLOBAL_STYLE}</style>
        <div class="app-shell">
            <main class="content">
                {move || match page.get() {
                    Page::Home => {
                        view! {
                            <HomePage
                                app_state=app_state
                                on_feed_now=move || set_page.set(Page::FeedNow)
                                on_open_schedule=move || set_page.set(Page::Schedule)
                            />
                        }
                            .into_any()
                    }
                    Page::FeedNow => {
                        view! {
                            <FeedNowPage
                                app_state=app_state
                                set_app_state=set_app_state
                                on_back=move || set_page.set(Page::Home)
                            />
                        }
                            .into_any()
                    }
                    Page::Schedule => {
                        view! {
                            <SchedulePage
                                app_state=app_state
                                set_app_state=set_app_state
                                on_back=move || set_page.set(Page::Home)
                            />
                        }
                            .into_any()
                    }
                    Page::Notifications => {
                        view! { <NotificationsPage app_state=app_state /> }.into_any()
                    }
                    Page::Settings => {
                        view! { <SettingsPage app_state=app_state set_app_state=set_app_state /> }
                            .into_any()
                    }
                }}
            </main>

            <Show when=move || !matches!(page.get(), Page::FeedNow | Page::Schedule)>
                <BottomNavigation current_page=page set_page=set_page />
            </Show>
        </div>
    }
}
