// pub mod playback_info {
//     use crossterm::{cursor, terminal, ExecutableCommand};
//     use serialport::{SerialPortSettings, DataBits, FlowControl, Parity, StopBits};
//     use std::io::{stdout, Write};
//     use std::time::Duration;

//     #[derive(Debug)]
//     struct Bitfield {
//         value: u8,
//     }

//     #[derive(Debug)]
//     struct U24(u32);

//     // 4bit単位のデータを扱うようにビットフィールドを作成
//     impl Bitfield {
//         // 2つの4-bit値で新しいBitfieldを作成
//         fn new(high: u8, low: u8) -> Self {
//             assert!(high <= 0x0F && low <= 0x0F, "Values must be 4-bit integers");
//             Self {
//                 value: (high << 4) | (low & 0x0F),
//             }
//         }

//         // 高位4-bit値を取得
//         fn high(&self) -> u8 {
//             (self.value >> 4) & 0x0F
//         }

//         // 低位4-bit値を取得
//         fn low(&self) -> u8 {
//             self.value & 0x0F
//         }
//     }

//     //24bit整数を扱うための
//     impl U24 {
//         fn from_be_bytes(high: u8, mid: u8, low: u8) -> Self {
//             Self(((high as u32) << 16) | ((mid as u32) << 8) | (low as u32))
//         }

//         fn value(&self) -> u32 {
//             self.0
//         }
//     }

//     pub fn process_event(port_name: String) -> Result<String, String> {
//         // シリアルポートの設定
//         let settings = SerialPortSettings {
//             baud_rate: 9600,
//             data_bits: DataBits::Eight,
//             flow_control: FlowControl::None,
//             parity: Parity::None,
//             stop_bits: StopBits::One,
//             timeout: Duration::from_millis(1500),
//         };

//         // シリアルポートを開く
//         let mut port = serialport::open_with_settings(&port_name, &settings)
//             .map_err(|e| format!("Failed to open serial port: {}", e))?;

//         // 音楽再生情報を受信するためのバッファ
//         let mut buffer = [0; 5]; // 最大5バイトのバッファ

//         /*
//         フラッシュ表示用の機能[flash]
//          */
//         // // keyとvelocity初期表示を設定
//         // let mut stdout = stdout();
//         // stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
//         // stdout.execute(cursor::MoveTo(0, 0)).unwrap();
//         // println!("Tempo: ");
//         // stdout.execute(cursor::MoveTo(0, 1)).unwrap();
//         // println!("Chanel: ");
//         // stdout.execute(cursor::MoveTo(0, 2)).unwrap();
//         // println!("Key: ");
//         // stdout.execute(cursor::MoveTo(0, 3)).unwrap();
//         // println!("Velocity: ");
//         // stdout.flush().unwrap();

//         loop {
//             // データを読み込む
//             match port.read_exact(&mut buffer) {
//                 Ok(_) => {
//                     // 受信したデータを16進数でログに表示
//                     println!("Received playback info (hex): {:02x?}", buffer);

//                     //let data_width = u8::from_le(buffer[0] & 0x0F);
//                     let flag_a = u8::from_le((buffer[1] >> 4) & 0x0F);
//                     let chanel = u8::from_le(buffer[1] & 0x0F);

//                     //flag_aの判定
//                     match flag_a {
//                         //key event
//                         0 => {
//                             // Little Endianであるため、bufferからkeyとvelocityを取り出す
//                             let key = u8::from_le(buffer[3]);
//                             let velocity = u8::from_le(buffer[4]);
//                             if velocity == 0{
//                                 //[flash]
//                                 // // カーソルを移動して値を上書き
//                                 // stdout.execute(cursor::MoveTo(8, 1)).unwrap(); // "Key: "の後に移動
//                                 // print!("{:2}, chanel");
//                                 // stdout.execute(cursor::MoveTo(5, 2)).unwrap(); // "Key: "の後に移動
//                                 // print!("{:6}", key); // 5桁の幅を確保して上書き
//                                 // stdout.execute(cursor::MoveTo(10, 3)).unwrap(); // "Velocity:"の後に移動
//                                 // print!("Noteoff"); // 10桁の幅を確保して上書き
//                                 // // バッファをフラッシュして表示を更新
//                                 // stdout.flush().unwrap();
//                                 println!("chanel: {}({:2}), key: {}({:6}), velocity: noteoff",
//                                     chanel, chanel, key, key);
//                             }else if velocity != 0{
//                                 //[flash]
//                                 // // カーソルを移動して値を上書き
//                                 // stdout.execute(cursor::MoveTo(8, 1)).unwrap(); // "Key: "の後に移動
//                                 // print!("{:2}", chanel);
//                                 // stdout.execute(cursor::MoveTo(5, 2)).unwrap(); // "Key: "の後に移動
//                                 // print!("{:6}", key); // 5桁の幅を確保して上書き
//                                 // stdout.execute(cursor::MoveTo(10, 3)).unwrap(); // "Velocity:"の後に移動
//                                 // print!("{:11}", velocity); // 3桁の幅を確保して上書き
//                                 // // バッファをフラッシュして表示を更新
//                                 // stdout.flush().unwrap();
//                                 println!("chanel: {}({:2}), key: {}({:6}), velocity: {}({:11})",
//                                     chanel, chanel, key, key, velocity, velocity);
//                             }
//                         },
//                         //tempo event
//                         1 => {
//                             let tempo = U24::from_be_bytes(buffer[2], buffer[3], buffer[4]);
//                             let bpm = 1000000 / tempo.value();

//                             //tempo情報を表示
//                             // stdout.execute(cursor::MoveTo(7, 0)).unwrap(); // "Tempo: "の後に移動
//                             // print!("{:?}", tempo);
//                             // stdout.flush().unwrap();
//                             println!("tempo: {:?}({:?})[μsec/四分音符], BPM: {}", tempo.value(), tempo, bpm);
//                         },
//                         //end event
//                         2 => {
//                             // 既存のchanel, key, velocity情報をクリア
//                             // stdout.execute(cursor::MoveTo(7, 0)).unwrap(); // "Tempo: "の後に移動
//                             // print!("   ");
//                             // stdout.execute(cursor::MoveTo(8, 1)).unwrap(); // "Chanel: "の後に移動
//                             // print!("   ");
//                             // stdout.execute(cursor::MoveTo(5, 2)).unwrap(); // "Key: "の後に移動
//                             // print!("   ");
//                             // stdout.execute(cursor::MoveTo(10, 3)).unwrap(); // "Velocity:"の後に移動
//                             // print!("   ");
//                             println!("End");
//                         },
//                         //nop event
//                         3 => {
//                             // No operation
//                         },
//                         //param event
//                         4 => {
//                             let event = u8::from_le((buffer[2] >> 4) & 0x0F);
//                             let slot = u8::from_le(buffer[2] & 0x0F);
//                             let param_data = u8::from_be(buffer[3]);

//                             match event {
//                                 0 => println!("Slot: {:6}, change param: {:11}", slot, param_data),
//                                 1 => println!("Detune/Multiple: {:6}, change param: {:11}", slot, param_data),
//                                 2 =>println!("TotalLevel: {:6}, change param: {:11}", slot, param_data),
//                                 3 => println!("KeyScale/AttackRate: {:6}, change param: {:11}", slot, param_data),
//                                 4 => println!("DecayRate: {:6}, change param: {:11}", slot, param_data),
//                                 5 => println!("SustainRate: {:6}, change param: {:11}", slot, param_data),
//                                 6 => println!("SustainLevel/ReleaseRate: {:6}, change param: {:11}", slot, param_data),
//                                 7 => println!("FeedBack/Connection: {:6}, change param: {:11}", slot, param_data),
//                                 _ => println!("Invalid event: {}", event),
//                             }
//                         },
//                         5 => println!("FlagA is 5: Skip to next track."),
//                         _ => println!("FlagA is invalid: {}", flag_a),
//                     }
//                 },
//                 Err(e) => {
//                     println!("Failed to read from serial port: {}", e);
//                 }
//             }
//         }
//     }
// }
