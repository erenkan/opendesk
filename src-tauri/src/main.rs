// Hide the Windows console window for release builds — GUI-only app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    opendesk_lib::run();
}
