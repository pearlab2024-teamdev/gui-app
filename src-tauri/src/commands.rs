// src/commands.rs
use tauri::{Window, State};
use serialport::{SerialPort, DataBits, Parity, StopBits, FlowControl};
use std::sync::{Arc, Mutex};
use crate::AppState;

#[tauri::command]
pub async fn set_serial_port(window: Window, port_name: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let baud_rate = 115200;

    let port = serialport::new(port_name.clone(), baud_rate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(std::time::Duration::from_millis(1500))
        .open()
        .map_err(|e| format!("Failed to open serial port: {}", e))?;

    let mut app_state = state.lock().unwrap();
    app_state.port = Some(Arc::new(Mutex::new(Box::new(port) as Box<dyn SerialPort>)));
    window.emit("playback_info", &format!("Serial port opened: {}", port_name)).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn disconnect_serial_port(window: Window, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut app_state = state.lock().unwrap();
    app_state.port = None;
    window.emit("playback_info", "Serial port disconnected").unwrap();
    Ok(())
}

// src/main.rs
// 変更なし
