use std::sync::Mutex;

use tauri::{ipc::Channel, State};

use crate::audio::{feature::AudioFeature, AudioHandler};

mod audio;

#[derive(Default)]
struct AppState {
    audio: Mutex<Option<AudioHandler>>,
}

#[tauri::command]
fn start_audio_stream(on_feature: Channel<AudioFeature>, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.audio.lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }
    let handler = AudioHandler::start(on_feature).map_err(|e| e.to_string())?;
    *guard = Some(handler);
    Ok(())
}

#[tauri::command]
fn stop_audio_stream(state: State<'_, AppState>) {
    let _ = state.audio.lock().unwrap().take();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![start_audio_stream, stop_audio_stream])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
