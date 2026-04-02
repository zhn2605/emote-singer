use std::{process::Command, sync::mpsc, thread, time::Duration};

use cpal::{Device, Devices, OutputDevices, Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ringbuf::{HeapRb, traits::{Consumer, Producer, Split}};

use crate::audio::{error::AudioError, feature::AudioFeature};

pub struct AudioHandler {
    pub device: Device,
    pub rx: mpsc::Receiver<AudioFeature>,
    stream: Stream,
}

impl AudioHandler {
    pub fn new() -> Result<Self, AudioError> {
        Self::builder(None)
    }

    pub fn new_with_device(device_name: &str) -> Result<Self, AudioError> {
        Self::builder(Some(device_name))
    }

    pub fn builder(device_name: Option<&str>) -> Result<Self, AudioError> {
        let host = cpal::host_from_id(cpal::HostId::Jack)
            .map_err(|_| AudioError::NoDevice)?;

        let monitor_name = Self::get_default_monitor()?;
        
        // defaulted to cpal_client_in for jack
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

        stream.play()?;
        thread::sleep(Duration::from_millis(300)); //temporary to ensure stream is up

        // disconnect mic & coinnect monitor directly
        Self::link_monitor_to_cpal(&monitor_name);

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

        Ok(Self { device, stream, rx })
    }

    fn link_monitor_to_cpal(monitor_sink: &str) {
        let monitor_fl = format!("{}:monitor_FL", monitor_sink);
    
        let links = Command::new("pw-link")
            .args(["--links"])
            .output()
            .map(|o| String::from_utf8(o.stdout).unwrap_or_default())
            .unwrap_or_default();
    
        // collect everything feeding into cpal_client_in
        let mut in_cpal_block = false;
        let mut to_disconnect: Vec<String> = Vec::new();
    
        for line in links.lines() {
            if line.starts_with("cpal_client_in") {
                in_cpal_block = true;
                continue;
            }
            if in_cpal_block {
                if line.contains("|<-") {
                    let src = line.trim().trim_start_matches("|<- ").to_string();
                    to_disconnect.push(src);
                } else {
                    in_cpal_block = false; // block ended
                }
            }
        }
    
        println!("Disconnecting from cpal: {:?}", to_disconnect);
    
        // disconnect all inputs (microphone)
        for src in to_disconnect {
            Command::new("pw-link")
                .args(["--disconnect", &src, "cpal_client_in:in_0"])
                .spawn().ok();
        }
    
        thread::sleep(Duration::from_millis(200));
    
        // connect monitor
        Command::new("pw-link")
            .args([&monitor_fl, "cpal_client_in:in_0"])
            .spawn().ok();

        println!("Linked {} -> cpal_client_in:in_0", monitor_fl);
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
        match host.input_devices() {
            Ok(devices) => Ok(devices),
            Err(e) => Err(AudioError::Device(e)),
        }
    }

    fn get_default_monitor() -> Result<String, AudioError> {
        let output = Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .map_err(|_| AudioError::NoDevice)?;
    
        Ok(String::from_utf8(output.stdout)
            .map_err(|_| AudioError::NoDevice)?
            .trim()
            .to_string())
    }
}