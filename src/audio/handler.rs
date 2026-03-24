use cpal::{Device, Devices, OutputDevices, Stream, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ringbuf::{HeapCons, HeapRb, traits::{Consumer, Producer, Split}};

use crate::audio::{error::AudioError, feature::AudioFeature};

pub struct AudioHandler {
    pub device: Device,
    stream: Stream,
    consumer: HeapCons<f32>,
    config: StreamConfig,
    feature: AudioFeature
}

impl AudioHandler {
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or(AudioError::NoDevice)?;

        let mut supported_configs_range = device.supported_output_configs()?;
        let supported_config = supported_configs_range.next()
            .ok_or(AudioError::NoNextConfig)?.with_max_sample_rate();

        let config = supported_config.config();

        // ring buff initialization 
        let rb  =HeapRb::<f32>::new(4096);
        let (mut producer, consumer) = rb.split();

        let stream = device.build_output_stream(
            &config, 
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data {
                    producer.try_push(*sample).ok();
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            }, 
            None
        )?;

        let feature = AudioFeature {
            rms: 0.0,
            zcr: 0.0
        };

        Ok(Self { device, stream, consumer, config, feature })
    }

    pub fn initialize_with_device() -> Result<Self, AudioError> {
        let host = cpal::default_host();

        // TODO: device selection based on str and id soon
        let device = host.default_output_device()
            .ok_or(AudioError::NoDevice)?;

        let mut supported_configs_range = device.supported_output_configs()?;
        let supported_config = supported_configs_range.next()
            .ok_or(AudioError::NoNextConfig)?.with_max_sample_rate();

        let config = supported_config.config();

        // ring buff initialization 
        let rb  =HeapRb::<f32>::new(4096);
        let (mut producer, consumer) = rb.split();

        let stream = device.build_output_stream(
            &config, 
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data {
                    producer.try_push(*sample).ok();
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            }, 
            None
        )?;

        let feature = AudioFeature {
            rms: 0.0,
            zcr: 0.0
        };

        Ok(Self { device, stream, consumer, config, feature })
    }

    pub fn start(&self) -> Result<(), AudioError> {
        self.stream.play()?;
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

    pub fn read_samples(&mut self, buf: &mut[f32]) -> usize {
        self.consumer.pop_slice(buf)
    }
    
}
