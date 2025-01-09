use std::sync::{Arc, Mutex};

use audio_manager::AudioStreamThreadManager;
use cpal::traits::{DeviceTrait, HostTrait};
use tauri::State;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod audio_manager;

#[tauri::command]
fn get_input_audio_devices() -> Vec<String> {
    let host = cpal::default_host();
    let devices = match host.input_devices(){
        Ok(devices) => devices,
        Err(_) => return vec![],
    };

    devices
        .filter_map(|device| device.name().ok())
        .collect()
}

#[tauri::command]
fn get_output_audio_devices() -> Vec<String> {
    let host = cpal::default_host();
    let devices = match host.output_devices() {
        Ok(devices) => devices,
        Err(_) => return vec![],
    };

    devices
        .filter_map(|device| device.name().ok())
        .collect()
}

#[tauri::command]
fn start_stream() -> Vec<String> {
    // TODO: impl start stream
    vec![]
}

#[tauri::command]
fn stop_stream(state: State<AppState>) -> Result<(), String> {
    let mut audio_manager = state.audio_stream_handle.lock().map_err(|_| "failed to lock audio manager".to_string())?;
    let audio_manager = audio_manager.as_mut().ok_or("audio manager has not init yet".to_string())?;

    audio_manager.stop()?;
    Ok(())
}

#[tauri::command]
fn set_input_device(input_device_name: String, state: State<AppState>) -> Result<(), String> {
    let mut audio_manager = state.audio_stream_handle.lock().map_err(|_| "failed to lock audio manager".to_string())?;
    let audio_manager = audio_manager.as_mut().ok_or("audio manager has not init yet".to_string())?;

    audio_manager.set_input_device(input_device_name)?;
    println!("set input device");
    Ok(())
}
#[tauri::command]
fn set_output_device(output_device_name: String, state: State<AppState>) -> Result<(), String> {
    let mut audio_manager = state.audio_stream_handle.lock().map_err(|_| "failed to lock audio manager".to_string())?;
    let audio_manager = audio_manager.as_mut().ok_or("audio manager has not init yet".to_string())?;

    audio_manager.set_output_device(output_device_name)?;
    println!("set output device");
    Ok(())
}

struct AppState {
    audio_stream_handle: Arc<Mutex<Option<AudioStreamThreadManager>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            audio_stream_handle: Arc::new(Mutex::new(Some(AudioStreamThreadManager::new())))
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_input_audio_devices,
            get_output_audio_devices,
            start_stream,
            stop_stream,
            set_input_device,
            set_output_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
