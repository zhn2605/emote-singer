use std::{process::Command, thread, time::Duration, sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, 
    Arc,
    },};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};
use tauri::ipc::Channel;

use crate::audio::{error::AudioError, feature::AudioFeature};

pub struct AudioHandler {
    shutdown: Arc<AtomicBool>,
    audio_thread: Option<thread::JoinHandle<()>>,
}

impl AudioHandler {
    pub fn start(channel: Channel<AudioFeature>) -> Result<Self, AudioError> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_inner = shutdown.clone();
        let (init_tx, init_rx) = mpsc::sync_channel::<Result<(), AudioError>>(1);
        let init_tx_thread = init_tx.clone();

        let audio_thread = thread::spawn(move || {
            if let Err(e) = Self::run(channel, shutdown_inner, &init_tx_thread) {
                let _ = init_tx_thread.send(Err(e));
            }
        });

        match init_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                shutdown,
                audio_thread: Some(audio_thread),
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AudioError::InitFailed),
        }
    }

    fn run(channel: Channel<AudioFeature>, shutdown: Arc<AtomicBool>, init_tx: &mpsc::SyncSender<Result<(), AudioError>>,) -> Result<(), AudioError> {
        let host = cpal::host_from_id(cpal::HostId::Jack).map_err(|_| AudioError::NoDevice)?;
        let monitor_name = Self::get_default_monitor()?;

        let device = host.default_input_device().ok_or(AudioError::NoDevice)?;

        let mut supported_configs_range = device.supported_input_configs()?;
        let supported_config = supported_configs_range
            .next()
            .ok_or(AudioError::NoNextConfig)?
            .with_max_sample_rate();
        let config = supported_config.config();

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
            None,
        )?;

        stream.play()?;
        Self::link_monitor_to_cpal(&monitor_name);

        let _ = init_tx.send(Ok(()));

        let mut buf = vec![0f32; 1024];
        let mut feature = AudioFeature { rms: 0.0, zcr: 0.0 };
        while !shutdown.load(Ordering::Relaxed) {
            let count = consumer.pop_slice(&mut buf);
            if count > 0 {
                feature.calculate(&buf[..count]);
                let _ = channel.send(feature);
            } else {
                thread::sleep(Duration::from_micros(500));
            }
        }

        drop(stream);
        Ok(())
    }

    fn link_monitor_to_cpal(monitor_sink: &str) {
        let monitor_fl = format!("{}:monitor_FL", monitor_sink);

        let links = Command::new("pw-link")
            .args(["--links"])
            .output()
            .map(|o| String::from_utf8(o.stdout).unwrap_or_default())
            .unwrap_or_default();

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
                    in_cpal_block = false;
                }
            }
        }

        println!("Disconnecting from cpal: {:?}", to_disconnect);

        for src in to_disconnect {
            Command::new("pw-link")
                .args(["--disconnect", &src, "cpal_client_in:in_0"])
                .spawn()
                .ok();
        }

        thread::sleep(Duration::from_millis(200));

        Command::new("pw-link")
            .args([&monitor_fl, "cpal_client_in:in_0"])
            .spawn()
            .ok();

        println!("Linked {} -> cpal_client_in:in_0", monitor_fl);
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

impl Drop for AudioHandler {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(t) = self.audio_thread.take() {
            let _ = t.join();
        }
    }
}
