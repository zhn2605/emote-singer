use std::{sync::mpsc, thread};

use cpal::{Device, Devices, OutputDevices, Stream, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ringbuf::{HeapRb, traits::{Consumer, Producer, Split}};

use crate::audio::{error::AudioError, feature::AudioFeature};

pub struct AudioHandler {
    pub device: Device,
    stream: Stream,
    config: StreamConfig,
    pub rx: mpsc::Receiver<AudioFeature>,
}

impl AudioHandler {
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();

        // TODO: Swap to monitor input
        let device = host.default_input_device()
            .ok_or(AudioError::NoDevice)?;

        let mut supported_configs_range = device.supported_input_configs()?;
        let supported_config = supported_configs_range.next()
            .ok_or(AudioError::NoNextConfig)?.with_max_sample_rate();

        let config = supported_config.config();

        // ring buff initialization 
        // prod is in audio call back, cons is in processing thread
        let rb = HeapRb::<f32>::new(4096);
        let (mut producer, mut consumer) = rb.split();
        
        let stream = device.build_input_stream(
            &config, 
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                producer.push_slice(data);
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            None
        )?;

        // tx goes to processing, rx returned in Self
        let (tx, rx) = mpsc::channel::<AudioFeature>();

        // processing thread
        thread::spawn(move || {
            let mut buf = vec![0f32; 1024];
            let mut feature = AudioFeature { rms: 0.0, zcr: 0.0 };
            loop {
                let count = consumer.pop_slice(&mut buf);
                if count > 0 {
                    feature.calculate(&buf[..count]);
                    tx.send(feature).ok();
                }
            }
        });

        Ok(Self { device, stream, config, rx })
    }

    pub fn play(&self) -> Result<(), AudioError> {
        self.stream.play()?;
        Ok(())
    }

    pub fn pause(&self) -> Result<(), AudioError> {
        self.stream.pause()?;
        Ok(())
    }

    // TODO: temporary testing
    pub fn return_devices(&self) -> Result<OutputDevices<Devices>, AudioError> {
        let host = cpal::default_host();
        match host.output_devices() {
            Ok(devices) => Ok(devices),
            Err(e) => Err(AudioError::Device(e)),
        }
    }    
}
