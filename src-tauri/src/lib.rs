use cpal::traits::{DeviceTrait, HostTrait};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_input_audio_devices,
            get_output_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
