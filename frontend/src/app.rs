use common::FeedingSchedule;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::bottom_navigation::BottomNavigation;
use crate::pages::{
    calendar::SchedulePage,
    feed::FeedNowPage,
    home::HomePage,
    notifications::NotificationsPage,
    settings::SettingsPage,
    setup::{
        AddFeederPage, BluetoothScanPage, ConnectionDetailsPage, FeederListPage, WifiSetupPage,
        WifiSetupQuestionPage,
    },
};
use crate::styles::Styles;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    FeederList,
    AddFeeder,
    BluetoothScan,
    WifiSetupQuestion,
    WifiSetup,
    ConnectionDetails,
    Home,
    FeedNow,
    Schedule,
    Notifications,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventTone {
    Success,
}

impl EventTone {
    pub fn class_name(self) -> &'static str {
        match self {
            Self::Success => "success",
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
pub struct FeederDevice {
    pub id: u32,
    pub name: String,
    pub connection: String,
    pub status: String,
    pub hopper_level: u8,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AppState {
    pub feeders: Vec<FeederDevice>,
    pub selected_feeder_id: Option<u32>,
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
            feeders: vec![
                FeederDevice {
                    id: 1,
                    name: String::from("Barsik feeder"),
                    connection: String::from("Wi-Fi"),
                    status: String::from("Online"),
                    hopper_level: 100,
                },
                FeederDevice {
                    id: 2,
                    name: String::from("Kitchen backup"),
                    connection: String::from("Bluetooth"),
                    status: String::from("Setup needed"),
                    hopper_level: 42,
                },
            ],
            selected_feeder_id: None,
            cat_name: String::from("Barsik"),
            cat_photo: String::from("assets/image.png"),
            feeder_connected: true,
            notifications_enabled: true,
            hopper_level: 100,
            last_fed_label: String::from("Today at 12:30 - 2 portions"),
            preferred_portion: 2,
            schedule: Vec::new(),
            schedule_loading: true,
            schedule_error: None,
            activity: Vec::new(),
            next_event_id: 1,
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

    pub fn select_feeder(&mut self, feeder_id: u32) {
        self.selected_feeder_id = Some(feeder_id);
        if let Some(feeder) = self.feeders.iter().find(|feeder| feeder.id == feeder_id) {
            self.hopper_level = feeder.hopper_level;
            self.feeder_connected = feeder.status == "Online";
        }
    }

    pub fn add_demo_feeder(&mut self, name: String, connection: String) -> u32 {
        let next_id = self
            .feeders
            .iter()
            .map(|feeder| feeder.id)
            .max()
            .unwrap_or(0)
            + 1;
        self.feeders.push(FeederDevice {
            id: next_id,
            name,
            connection,
            status: String::from("Online"),
            hopper_level: 100,
        });
        next_id
    }

    pub fn clear_selected_feeder(&mut self) {
        self.selected_feeder_id = None;
        self.schedule.clear();
        self.schedule_loading = false;
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
                    state.set_schedule_error(error);
                });
            }
        }
    });
}

fn log_client_action(action: &'static str, detail: Option<String>) {
    crate::tauri_api::report_button_click(action, detail.clone());
    spawn_local(async move {
        let _ = api::track_client_action(action, detail).await;
    });
}

#[component]
pub fn App() -> impl IntoView {
    let (page, set_page) = signal(Page::FeederList);
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
                    Page::FeederList => {
                        view! {
                            <FeederListPage
                                app_state=app_state
                                set_app_state=set_app_state
                                on_add=move || set_page.set(Page::AddFeeder)
                                on_open=move || {
                                    log_client_action("feeder_selected", None);
                                    sync_schedules(set_app_state);
                                    set_page.set(Page::Home);
                                }
                            />
                        }
                            .into_any()
                    }
                    Page::AddFeeder => {
                        view! {
                            <AddFeederPage
                                on_back=move || set_page.set(Page::FeederList)
                                on_bluetooth=move || set_page.set(Page::BluetoothScan)
                                on_wifi=move || set_page.set(Page::ConnectionDetails)
                            />
                        }
                            .into_any()
                    }
                    Page::BluetoothScan => {
                        view! {
                            <BluetoothScanPage
                                on_back=move || set_page.set(Page::AddFeeder)
                                on_continue=move || set_page.set(Page::WifiSetupQuestion)
                            />
                        }
                            .into_any()
                    }
                    Page::WifiSetupQuestion => {
                        view! {
                            <WifiSetupQuestionPage
                                on_back=move || set_page.set(Page::BluetoothScan)
                                on_yes=move || set_page.set(Page::WifiSetup)
                                on_skip=move || set_page.set(Page::Home)
                            />
                        }
                            .into_any()
                    }
                    Page::WifiSetup => {
                        view! {
                            <WifiSetupPage
                                on_back=move || set_page.set(Page::WifiSetupQuestion)
                                on_connected=move || set_page.set(Page::ConnectionDetails)
                            />
                        }
                            .into_any()
                    }
                    Page::ConnectionDetails => {
                        view! {
                            <ConnectionDetailsPage
                                set_app_state=set_app_state
                                on_back=move || set_page.set(Page::AddFeeder)
                                on_done=move || set_page.set(Page::Home)
                            />
                        }
                            .into_any()
                    }
                    Page::Home => {
                        view! {
                            <HomePage
                                app_state=app_state
                                on_feed_now=move || {
                                    log_client_action("open_feed_now", None);
                                    set_page.set(Page::FeedNow);
                                }
                                on_open_schedule=move || {
                                    log_client_action("open_schedule", None);
                                    set_page.set(Page::Schedule);
                                }
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
                        view! {
                            <SettingsPage
                                app_state=app_state
                                set_app_state=set_app_state
                                on_feeder_list=move || set_page.set(Page::FeederList)
                            />
                        }
                            .into_any()
                    }
                }}
            </main>

            <Show when=move || matches!(page.get(), Page::Home | Page::Notifications | Page::Settings)>
                <BottomNavigation current_page=page set_page=set_page />
            </Show>
        </div>
    }
}
