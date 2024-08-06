// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serialport::{DataBits, FlowControl, Parity, SerialPort, SerialPortSettings, StopBits};
use std::io::{self, Read, Write};
use std::time::Duration;
use tauri::Manager;
use tauri::Window;
use tauri::State;
use std::sync::{Arc, Mutex};
//use magical_global as magical

// Ymodem関連の定数
const SOH: u8 = 0x01; // Start Of Header
const STX: u8 = 0x02; // Start Of Text
const EOT: u8 = 0x04; // End Of Transmission
const ACK: u8 = 0x06; // Acknowledge
const NAK: u8 = 0x15; // Negative Acknowledge
const C: u8 = 0x43; // 'C' for CRC mode

/// ファイルをYmodemプロトコルで送信する関数
///
/// # Arguments
///
/// * `contents` - 送信するファイルのバイト列
/// * `settings` - シリアルポートの設定
/// * `port_name` - シリアルポートの名前
///
/// # Returns
///
/// `io::Result<()>` - 送信が成功した場合はOk(()), エラーが発生した場合はエラーを返す。
fn ymodem_file_send(
    contents: &[u8],
    _settings: &SerialPortSettings,
    port: &mut Box<dyn SerialPort> ,
) -> io::Result<()> {
    // シリアルポートを開く
    // let mut port = serialport::open_with_settings(port_name, settings)?;

    // 受信側からの 'C' 信号を待つ
    let mut response = [0; 1];
    loop {
        port.read_exact(&mut response)?;
        if response[0] == C {
            break;
        }
    }

    // ファイルヘッダの送信
    let file_header = create_file_header("example.mid", contents.len() as u64)?;
    port.write_all(&file_header)?;

    // ACKを待つ
    wait_for_ack(&mut *port)?;

    // ファイルデータの送信
    let mut block_number = 0; // ブロック番号は0から開始
    for chunk in contents.chunks(128) {
        let data_block = create_data_block(chunk, block_number +1)?;
        port.write_all(&data_block)?;

        // ACKを待つ
        wait_for_ack(&mut *port)?;

        block_number += 1;
    }

    // EOTの送信
    port.write_all(&[EOT])?;
    wait_for_ack(&mut *port)?;
    let data_block = create_data_block(&vec![0;128], 0)?;
    port.write_all(&data_block)?;
    // 最後のACKを待つ
    wait_for_ack(&mut *port)?;
    println!("YMODEM PASS!");
    Ok(())
}

/// ファイルのファイルヘッダを作成する関数
///
/// # Arguments
///
/// * `filename` - ファイル名
/// * `filesize` - ファイルのサイズ
///
/// # Returns
///
/// `io::Result<Vec<u8>>` - ファイルヘッダのバイト列を含む結果。エラーが発生した場合はエラーを返す。
fn create_file_header(filename: &str, filesize: u64) -> io::Result<Vec<u8>> {
    let mut header = vec![SOH, 0, 255];
    let mut file_info = Vec::new();
    file_info.extend_from_slice(filename.as_bytes());
    file_info.push(0); // null terminator
    file_info.extend_from_slice(filesize.to_string().as_bytes());
    file_info.push(0); // null terminator

    let mut block = vec![0u8; 128];
    block[..file_info.len()].copy_from_slice(&file_info);
    header.extend_from_slice(&block);
    let crc_value = crc16_ccitt(&block);
    header.push((crc_value >> 8) as u8);
    header.push((crc_value & 0xFF) as u8);

    Ok(header)
}

/// データブロックを作成する関数
///
/// # Arguments
///
/// * `chunk` - 送信するデータのバイト列
/// * `block_number` - データブロックの番号
///
/// # Returns
///
/// `io::Result<Vec<u8>>` - データブロックのバイト列を含む結果。エラーが発生した場合はエラーを返す。
fn create_data_block(chunk: &[u8], block_number: u8) -> io::Result<Vec<u8>> {
    let mut block = vec![ SOH/*STX*/, dbg!(block_number), !block_number];
    let mut data = vec![0u8; 128];
    data[..chunk.len()].copy_from_slice(chunk);
    block.extend_from_slice(&data);

    // Convert CRC value to little-endian
    let crc_value = crc16_ccitt(&data);
    let crc_bytes = crc_value.to_le_bytes();

    block.push((crc_value >> 8) as u8);
    block.push((crc_value & 0xFF) as u8);

    Ok(block)
}

/// ACKを待つ関数
///
/// # Arguments
///
/// * `port` - シリアルポート
///
/// # Returns
///
/// `io::Result<()>` - ACKを受信した場合はOk(()), エラーが発生した場合はエラーを返す。
fn wait_for_ack(port:&mut Box<dyn SerialPort>) -> io::Result<()> {
    let mut response = [0; 1];
    loop {
        port.read_exact(&mut response)?;
        if response[0] == ACK {
            break;
        }
    }
    Ok(())
}

/// CRC-16-CCITTを計算する関数
///
/// # Arguments
///
/// * `data` - CRCを計算するデータのバイト列
///
/// # Returns
///
/// `u16` - 計算されたCRC値
fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

//MISI形式のファイルか判定する関数
fn check_midi_format(contents: &[u8]) -> bool {
    contents.starts_with(b"MThd")
}


// エラーメッセージを格納する構造体
#[derive(serde::Serialize)]
struct ErrorMessage {
    error: String,
}

// ファイル情報を格納する構造体
#[derive(serde::Serialize)]
struct FileInfo {
    size: usize,
    is_midi: bool,
}

//アプリケーションの状態を保持するための構造体
#[derive(serde::Serialize)]
struct AppState;

#[derive(Debug)]
struct U24(u32);

//24bit整数を扱うための
impl U24 {
    fn from_be_bytes(high: u8, mid: u8, low: u8) -> Self {
        Self(((high as u32) << 16) | ((mid as u32) << 8) | (low as u32))
    }

    fn value(&self) -> u32 {
        self.0
    }
}


// ファイルの内容を受け取り、情報を返すTauriコマンド
// #[tauri::command]
// fn read_file(contents: Vec<u8>, state: State<'_, Arc<Mutex<AppState>>>) -> Result<FileInfo, String> {
//     println!("Reading file with contents of length: {}", contents.len()); // デバッグ用ログ

//     // MIDIファイルかどうかを確認
//     let is_midi = contents.len() >= 4 && &contents[..4] == b"MThd";
//     println!("File size: {}, Is MIDI: {}", contents.len(), is_midi); // デバッグ用ログ
//     Ok(FileInfo {
//         size: contents.len(),
//         is_midi,
//     })
// }

//ファイルサイズと形式を判定するtauriコマンド
#[tauri::command]
fn read_file(contents: Vec<u8>, state: State<'_, Arc<Mutex<AppState>>>) -> Result<FileInfo, String> {
    println!("Reading file with contents of length: {}", contents.len()); // デバッグ用ログ

    let size = contents.len();
    let is_midi = check_midi_format(&contents);

    // if is_midi {
    //     //stateをロックしてmidi_file_sentを更新
    //     let mut app_state = state.lock().unwrap();
    //     app_state.midi_file_sent = true;
    // }   

    Ok(FileInfo { size, is_midi})
}

// ファイルサイズをシリアル通信で送信するTauriコマンド
#[tauri::command]
async fn send_file_size<'a>(window: Window, contents: Vec<u8>, port_name: String, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    //ファイルがMIDI形式かどうかを確認
    if !check_midi_format(&contents) {
        //MIDI形式でない場合returnエラー
        return Err("You choosed not MIDI file".into());
    }

    // ファイル情報を取得
    let file_info = read_file(contents.clone(), state)?;

    // シリアルポートの設定
    let settings = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1500),
    };

    // シリアルポートを開く
    let mut port = serialport::open_with_settings(&port_name, &settings)
        .map_err(|e| format!("Failed to open serial port: {}", e))?;
    println!("Serial port opened: {}", port_name);

    // ファイルサイズをリトルエンディアンでバイト配列に変換
    let size_bytes = file_info.size.to_le_bytes();
    println!("file byte size: {:?}", size_bytes);

    //let bit4_header = Bitfield::new(0x0F, 0x02);
    let bit4_header = 0x2F; //リトルエンディアンに対応させる
    let all_data: [u8; 4] = [bit4_header, size_bytes[0], size_bytes[1], size_bytes[2]];

    // シリアルポートにデータを書き込む
    port.write_all(&all_data)
        .map_err(|e| format!("Failed to write to serial port: {}", e))?;
    println!("File size sent!");

    //データ送信が始まったことを知らせるイベント
    println!("Starting send file!");
    window.emit("playback_info", &"Starting send file!").unwrap();

    // シーケンサからの受信可能メッセージを待機
    let mut response = [0; 1];
    match port.read_exact(&mut response) {
        Ok(_) => {
            let message = format!("Received response byte: {:02x}", response[0]);
            println!("{}", message);
            window.emit("playback_info", message).unwrap();

            let high_resp = (response[0] >> 4) & 0x0F;
            let low_resp = (response[0] & 0x0F);
            println!("High nibble: {:x}, Low nibble: {:x}", high_resp, low_resp);

            if high_resp == 0x0 && low_resp == 0xE {
                // ファイルデータをシリアルポートに書き込む
                // port.write_all(&contents)
                //     .map_err(|e| format!("Failed to send file data: {}", e))?;
                //ymodem形式でファイル送信
                ymodem_file_send(&contents, &settings, &mut port)
                    .map_err(|e| format!("Failed to send file using Ymodem: {}", e))?;


                // シーケンサからの受信完了メッセージを待機
                let mut ack = [0; 1];
                match port.read_exact(&mut ack) {
                    Ok(_) => {
                        println!("Received ack byte: {:02x}", ack[0]);
                        window.emit("playback_info", &("Received ack byte: {:02x}", ack[0])).unwrap();
                        window.emit("playback_info", &("File transfer successful, checksum: {:?}", ack[0])).unwrap();

                        let ack_high_nibble = (ack[0] >> 4) & 0x0F;
                        let ack_low_nibble = ack[0] & 0x0F;
                        let ack_ok: [u8; 1] = [0xB0];
                        let ack_err: [u8; 1] = [0xA0];

                        //受信完了メッセージのヘッダ情報かつチェックサムの内容が一致しているか
                        if ack_high_nibble == 0xD && ack_low_nibble == 0x0 {
                            //データ転送終了メッセ
                            let ack_message = format!("Received ack byte: {:02x}", ack[0]);
                            println!("{}", ack_message);
                            window.emit("playback_info", ack_message).unwrap();
                            //window.emit("playback_info", &("File transfer successful, checksum: {:?}", ack[0])).unwrap();
                        } else if ack_low_nibble == 0xC {
                            //異常終了メッセ
                            port.write_all(&ack_err)
                                .map_err(|e| format!("Failed to write to serial port: {}", e))?;
                            window.emit("playback_info", &"Failed to write to serial port: {}").unwrap();
                            return Err("Incomplete file transfer".into());
                        }
                    }
                    Err(e) => {
                        println!("Failed to read ack from serial port: {}", e);
                        // タイムアウト後の処理として未完了メッセージを送信する
                        port.write_all(&[0xC0])
                            .map_err(|e| format!("Failed to send incomplete message: {}", e))?;
                        window.emit("playback_info", &"Failed to read ack from serial port").unwrap();
                    }
                }
            } else {
                return Err("Sequencer not ready".into());
            }
        }
        Err(e) => {
            println!("Failed to read from serial port: {}", e);
            // タイムアウト後の処理として未完了メッセージを送信する
            port.write_all(&[0xC0])
                .map_err(|e| format!("Failed to send incomplete message: {}", e))?;
            window.emit("playback_info", &"Failed to read from serial port").unwrap();
        }
    }

    // 音楽再生情報を受信するためのバッファ
    let mut buffer = [0; 5]; // 最大5バイトのバッファ
    
    // フロントへのメッセージ送信デモ
    window.emit("playback_info", &"Starting playback info").unwrap();

    loop {
        let mut msg_combined = 0u64;
        match port.read_exact(&mut buffer[0..1]) {
          Ok(_) => {
            assert!(buffer[0] & 0x0F == 0x01);
            msg_combined = buffer[0] as u64;
          },
          Err(e) => {
            println!("Failed to read from serial port: {}", e);
            // エラーメッセージを作成
            let error_message = ErrorMessage {
                error: format!("Failed to read from serial port: {}", e),
            };

            // JSON形式でフロントエンドにメッセージ送信
            window.emit("playback_info", &error_message).unwrap();
          }
        }
        let following_size = (buffer[0] >> 4) & 0xF;
        if following_size == 0 {
          let flaga_msg = "End".to_string(); 
          println!("{}", flaga_msg);
          window.emit("playback_info", &flaga_msg).unwrap();
          break;
        }
        println!("first byte: {:02X}", buffer[0]);
        println!("size: {}", following_size);
        println!("size: {}", following_size);
        assert!(following_size < 5);
        let mut tmp_buffer = vec![0; following_size as usize];
        // データを読み込む
        match port.read_exact(&mut tmp_buffer) {
            Ok(_) => {
                for i in 0..(following_size as usize) {
                  buffer[i + 1] = tmp_buffer[i];
                  msg_combined |= (buffer[i + 1] as u64) << ((i + 1) * 8);
                }

                // 受信したデータを16進数でログに表示
                println!("Received playback info (hex): {:02x?}", &buffer[0..(following_size as usize + 1)]);

                // JSON形式での送信を想定してデータを変換
                // let data_to_send = serde_json::to_string(&buffer[0..(following_size as usize + 1)])
                //    .map_err(|e| e.to_string())?;

                // フロントエンドにメッセージを送信
                // window.emit("playback_info", &data_to_send).unwrap();

                //let data_width = u8::from_le(buffer[0] & 0x0F);
                let flag_a = u8::from_le((buffer[1]) & 0x0F);
                let chanel = u8::from_le(buffer[1]>> 4 & 0x0F);

                let tauri_msg = [(msg_combined & 0xFFFFFFFF) as u32, (msg_combined >> 32) as u32];

                //flag_aの判定
                match flag_a {
                    //key event
                    0 => {
                        // Little Endianであるため、bufferからkeyとvelocityを取り出す
                        let key = u8::from_le(buffer[2]);
                        let velocity = u8::from_le(buffer[3]);

                        if velocity == 0 {
                            let flaga_msg = format!("chanel: {}({:2}), key: {}({:6}), velocity: 0({:11})",
                                chanel, chanel, key, key, 0);
                            println!("{}", flaga_msg);
                            window.emit("playback_info", &tauri_msg).unwrap();
                        }else if velocity != 0{
                            let flaga_msg = format!("chanel: {}({:2}), key: {}({:6}), velocity: {}({:11})",
                                chanel, chanel, key, key, velocity, velocity);
                            println!("{}", flaga_msg);
                            window.emit("playback_info", &tauri_msg).unwrap();
                        }
                    }
                    //tempo event
                    1 => {
                        let tempo = U24::from_be_bytes(buffer[2], buffer[3], buffer[4]);
                        let bpm = 1000000 / tempo.value();

                        let flaga_msg = format!("tempo: {:?}({:?})[μsec/四分音符], BPM: {}", tempo.value(), tempo, bpm);
                        println!("{}", flaga_msg);
                        window.emit("playback_info", &tauri_msg).unwrap();
                    },
                    //end event
                    2 => {
                        unreachable!();
                    },
                    //nop event
                    3 => {
                        // No operation
                    }
                    //param event
                    4 => {
                        let event = u8::from_le((buffer[2] >> 4) & 0x0F);
                        let slot = u8::from_le(buffer[2] & 0x0F);
                        let param_data = u8::from_be(buffer[3]);

                        let flaga_msg = match event {
                            0 => format!("Slot: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            1 => format!("Detune/Multiple: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            2 => format!("TotalLevel: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            3 => format!("KeyScale/AttackRate: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            4 => format!("DecayRate: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            5 => format!("SustainRate: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            6 => format!("SustainLevel/ReleaseRate: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            7 => format!("FeedBack/Connection: {}({:6}), change param: {}({:11})", slot, slot, param_data, param_data),
                            _ => format!("Invalid event: {}", event),
                        };

                        println!("{}", flaga_msg);
                        // window.emit("playback_info", &tauri_msg).unwrap();
                    },
                    5 => {
                        let flaga_msg = "FlagA is 5: Skip to next track.".to_string();
                        println!("{}", flaga_msg);
                        window.emit("playback_info", &tauri_msg).unwrap();
                    }
                    _ => {
                        let flaga_msg = format!("FlagA is invalid: {}", flag_a);
                        println!("{}", flaga_msg);
                        window.emit("playback_info", &tauri_msg).unwrap();
                    }
                }
            }
            Err(e) => {
                println!("Failed to read from serial port: {}", e);
                // エラーメッセージを作成
                let error_message = ErrorMessage {
                    error: format!("Failed to read from serial port: {}", e),
                };

                // JSON形式でフロントエンドにメッセージ送信
                window.emit("playback_info", &error_message).unwrap();
            }
        }
    }

    Ok(())
}


// アプリケーションのエントリーポイント
fn main() {
    //MIDI判定の状態管理
    let app_state = Arc::new(Mutex::new(AppState));

    // ignore proxy
    let proxy_env_value = match std::env::var("http_proxy") {
        Ok(val) => {
            std::env::set_var("http_proxy", "");
            std::env::set_var("https_proxy", "");
            val
        }
        Err(e) => String::from("proxy setting error"),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            read_file,
            send_file_size, // 本番用
            //send_file_test  // テスト用
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // let port_name = "COM3".to_string();
    // process_event(port_name);
    // reset env proxy

    if !proxy_env_value.is_empty() {
        std::env::set_var("http_proxy", proxy_env_value.as_str());
        std::env::set_var("https_proxy", proxy_env_value.as_str());
    }
}