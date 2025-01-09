use std::{any::Any, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Device, SampleRate, StreamConfig, SupportedStreamConfig};
use ringbuf::{traits::{Consumer, Producer, Split}, HeapRb};
use tauri::State;

const AUDIO_STREAMER_INTERVAL_MILLIS: u64 = 15;
const AUDIO_STREAMER_TIMEOUT_MILLIS: u64 = 10000;

pub struct Stopper {
    stop_flg: Arc<AtomicBool>,
}
impl Stopper {
    fn new() -> Self {
        Stopper {
            stop_flg: Arc::new(AtomicBool::new(false))
        }
    }
    pub fn send_stop_signal(&self) {
        self.stop_flg.store(true, Ordering::Relaxed);
    }
}

pub struct AudioStreamThreadManager {
    input_device: Option<Device>,
    input_config: Option<SupportedStreamConfig>,
    output_device: Option<Device>,
    output_config: Option<SupportedStreamConfig>,
    handle: Option<JoinHandle<Result<(),()>>>,
    stopper: Option<Stopper>,
}

impl AudioStreamThreadManager {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let input_device = host.default_input_device();
        let input_config = match input_device.as_ref() {
            Some(device) => device.default_input_config().ok(),
            None => None
        };
        let output_device = host.default_output_device();
        let output_config = match output_device.as_ref() {
            Some(device) => device.default_output_config().ok(),
            None => None
        };
        AudioStreamThreadManager{
            input_device,
            input_config,
            output_device,
            output_config,
            handle: None,
            stopper: None
        }
    }
    pub fn run(
        &mut self, 
        latency: f32,
        sample_rate: u32,
        channels: u16,
        buffer_size: u32,
    ) -> Result<(), String> {
        // デバイス設定
        let input_device = self.input_device.clone().ok_or("input device is not found".to_string())?;
        let output_device = self.output_device.clone().ok_or("output device is not found".to_string())?;

        // 仮実装 input_configをoutput_configにも使用
        let stream_config = StreamConfig {
            channels,
            sample_rate: SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(buffer_size)
        };

        // 遅延調整
        let latency_frames: f32 = (latency / 1_000.0) * stream_config.sample_rate.0 as f32;
        let latency_samples: usize = latency_frames as usize * stream_config.channels as usize;

        let ring = HeapRb::<f32>::new(latency_samples * 2);
        let (mut producer, mut consumer) = ring.split();
        producer.push_slice(vec![0.0f32; latency_samples].as_slice());

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut output_fell_behind = false;
            for &sample in data {
                if producer.try_push(sample).is_err() {
                    output_fell_behind = true;
                }
            }
            if output_fell_behind {
                eprintln!("output stream fell behind: try increasing latency");
            }
        };
        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut input_fell_behind = false;
            for sample in data {
                *sample = match consumer.try_pop() {
                    Some(s) => s,
                    None => {
                        input_fell_behind = true;
                        0.0
                    }
                };
            }
            if input_fell_behind {
                eprintln!("input stream fell behind: try increasing latency");
            }
        };

        let stopper = Stopper::new();
        let stop_flg_clone = Arc::clone(&stopper.stop_flg);

        // StreamThread生成
        let handle = thread::spawn(move || {
            let input_stream = input_device
                .build_input_stream(&stream_config, input_data_fn, err_fn, None)
                // .or(Err("failed to build input stream".to_string()))?;
                .or(Err(()))?;

            let output_stream = input_device
                .build_output_stream(&stream_config, output_data_fn, err_fn, None)
                // .or(Err(()))?;
                .or(Err(()))?;
            input_stream.play().or(Err(()))?;
            output_stream.play().or(Err(()))?;
            while stop_flg_clone.load(std::sync::atomic::Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(AUDIO_STREAMER_INTERVAL_MILLIS));
            }
            Ok(())
        });

        self.handle = Some(handle);
        self.stopper= Some(stopper);
        Ok(())
    }
    pub fn stop(&self) -> Result<(), String> {
        match (self.handle.as_ref(), self.stopper.as_ref()) {
            // 実行中に限り、ストップ信号送信
            (Some(handle), Some(stopper)) => {
                let mut timeout_count: u64 = 0;
                stopper.send_stop_signal();
                while handle.is_finished() {
                    if timeout_count > AUDIO_STREAMER_TIMEOUT_MILLIS {
                        return Err("stream thread couldn't stop".to_string());
                    }
                    thread::sleep(std::time::Duration::from_millis(AUDIO_STREAMER_INTERVAL_MILLIS));
                    timeout_count += AUDIO_STREAMER_INTERVAL_MILLIS;
                };
                return Ok(());
            },
            (None, None) => {
                return Ok(());
            },
            (_, _) => {
                return Err("stream thread's state is invalidate".to_string());
            }
        }
    }
    pub fn set_input_device(&mut self, input_device_name: String) -> Result<(), String> {
        let host = cpal::default_host();
        let input_devices = host
            .input_devices()
            .map_err(|e| format!("Failed to get input devices: {}", e))?;

        // デバイス名でフィルタリング
        for device in input_devices {
            if let Ok(name) = device.name() {
                if name == input_device_name {
                    self.input_device = Some(device.clone());
                    return Ok(());
                }
            }
        }
        self.input_device = None;
        Err(format!("Input device '{}' not found", input_device_name))
    }
    pub fn set_output_device(&mut self, output_device_name: String) -> Result<(), String> {
        let host = cpal::default_host();
        let output_devices = host
            .output_devices()
            .map_err(|e| format!("Failed to get output devices: {}", e))?;

        // デバイス名でフィルタリング
        for device in output_devices {
            if let Ok(name) = device.name() {
                if name == output_device_name {
                    self.output_device = Some(device.clone());
                    return Ok(());
                }
            }
        }
        self.output_device = None;
        Err(format!("Output device '{}' not found", output_device_name))
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}