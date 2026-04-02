// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cpal::traits::{DeviceTrait, HostTrait};

mod audio;

fn main() {
    let audio_handler = match audio::AudioHandler::new().expect("audio init failed");
    let rx = audio_handler.rx;

    // println!("Default Input device: {} {}", audio_handler.device.description().unwrap().name(), audio_handler.device.id().unwrap());

    // loop {
    //     if let Ok(feature) = audio_handler.rx.recv() {
    //         println!("RMS: {:.4}, ZCR: {:.4}", feature.rms, feature.zcr);
    //     }
    // }

    app_lib::run();
}
