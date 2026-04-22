pub mod audio;

use std::sync::mpsc::Receiver;
use tauri::Emitter;

use crate::audio::{AudioHandler, feature::AudioFeature};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(audio: AudioHandler, rx: Receiver<AudioFeature>) {
    tauri::Builder::default()
        .setup(move |app| {
            // Own the AudioHandler (and its !Send-safe Stream) on a dedicated
            // thread that simply keeps it alive for the lifetime of the app.
            // Dropping the Stream would stop audio capture.
            std::thread::spawn(move || {
                let _keep_alive = audio;
                std::thread::park();
            });

            // Drain features from the processing thread and forward them to
            // the frontend as Tauri events. This is the push path that drives
            // image animation (stretch / jiggle / emoji swap).
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                while let Ok(feature) = rx.recv() {
                    handle.emit("audio-feature", feature).ok();
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
