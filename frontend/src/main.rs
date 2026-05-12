mod api;
mod app;
mod components;
mod pages;
mod styles;
mod tauri_api;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
