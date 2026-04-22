// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let (handler, rx) = match app_lib::audio::AudioHandler::new() {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("Failed to initialize audio handler: {}", e);
            return;
        }
    };

    app_lib::run(handler, rx);
}
