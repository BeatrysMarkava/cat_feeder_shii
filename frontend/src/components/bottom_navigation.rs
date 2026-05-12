use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::app::Page;
use crate::tauri_api;

#[component]
pub fn BottomNavigation(
    current_page: ReadSignal<Page>,
    set_page: WriteSignal<Page>,
) -> impl IntoView {
    let nav_button = move |icon_src: &'static str, target: Page, icon_class: &'static str| {
        let action = match target {
            Page::Home => "open_home",
            Page::Notifications => "open_notifications",
            Page::Settings => "open_settings",
            _ => "open_page",
        };

        view! {
            <button
                class=("nav-btn", true)
                class:active=move || current_page.get() == target
                on:click=move |_| {
                    tauri_api::report_button_click(action, None);
                    spawn_local(async move {
                        let _ = api::track_client_action(action, None).await;
                    });
                    set_page.set(target);
                }
            >
                <img src=icon_src alt="" class=format!("nav-icon {icon_class}") />
            </button>
        }
    };

    view! {
        <nav class="bottom-navigation">
            {nav_button("assets/main_bottom_bar.png", Page::Home, "nav-icon-home")}
            {nav_button(
                "assets/notification_bottom_bar.png",
                Page::Notifications,
                "nav-icon-notifications",
            )}
            {nav_button("assets/setting_bottom_bar.png", Page::Settings, "nav-icon-settings")}
        </nav>
    }
}
