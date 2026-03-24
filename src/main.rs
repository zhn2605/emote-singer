mod audio;

use cpal::traits::DeviceTrait;

fn main() {
    let audio_handler = match audio::AudioHandler::new() {
        Ok(handler) => handler,
        Err(e) => {
            eprintln!("Failed to initialize audio handler: {}", e);
            return;
        }
    };

    let devices = match audio_handler.return_devices() {
        Ok(devices) => devices,
        Err(e) => {
            eprintln!("Failed to retrieve output devices: {}", e);
            return;
        }
    };

    for device in devices {
        let device_description = device.description().unwrap();
        let device_name = device_description.name();
        let device_id = device.id().unwrap();
        println!("Output device: {} {}", device_name, device_id);
    }

    println!("Default output device: {} {}", audio_handler.device.description().unwrap().name(), audio_handler.device.id().unwrap());

    audio_handler.start().unwrap();

    
}

