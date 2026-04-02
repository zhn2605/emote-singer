// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cpal::traits::{DeviceTrait, HostTrait};

mod audio;

fn main() {
    let audio_handler = match audio::AudioHandler::new() {
        Ok(handler) => handler,
        Err(e) => {
            eprintln!("Failed to initialize audio handler: {}", e);
            return;
        }
    };
    let rx = audio_handler.rx;

    app_lib::run();
}
