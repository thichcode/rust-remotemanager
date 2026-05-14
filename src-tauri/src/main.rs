// Prevents the Tauri app from opening a console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hermes_remote_manager_lib::run()
}
