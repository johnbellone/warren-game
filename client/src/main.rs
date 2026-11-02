// Prevents additional console windows on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialise logging so [IPC] messages appear in stdout/terminal.
    simple_logger::init_with_env().ok();
    warren_client::run();
}
