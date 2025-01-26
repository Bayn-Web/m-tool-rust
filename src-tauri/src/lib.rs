use serde::Deserialize;
use serde_json::{self, json};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Manager, Window};
mod music_player;
use rodio::Sink;
use serde::Serialize;

#[derive(Serialize)]
pub struct Song {
    pub label: String,
    pub value: String,
}
struct AppState {
    current_song: Mutex<Option<Arc<Sink>>>,
}
// getJsonData 读取一个本地json文件
#[tauri::command]
fn get_json_data() -> String {
    // 判断文件是否存在
    let file_path = "./data.json";
    if !Path::new(file_path).exists() {
        // 选择创建一个默认的 JSON 文件
        let default_json = r#"[]"#;
        File::create(file_path)
            .and_then(|mut f| f.write_all(default_json.as_bytes()))
            .expect("Failed to create default JSON file");
        // 再次尝试读取文件
        fs::read_to_string(file_path).expect("Failed to read file after creation")
    } else {
        fs::read_to_string(file_path).expect("Failed to read existing file")
    }
}

// uploadFilePath 追加文件路径
#[tauri::command]
fn upload_file_path(path: String) -> String {
    let file_path = "./data.json";
    // 选择创建一个默认的 JSON 文件
    let json_string = fs::read_to_string(file_path).expect("Failed to read existing file");
    let mut json_array: Vec<serde_json::Value> = serde_json::from_str(&json_string)
        .map_err(|e| format!("Failed to parse JSON string: {}", e))
        .unwrap();
    let new_object = json!({
        "label": path,
        "value": Path::new(&path).file_name().and_then(|f| f.to_str()).unwrap_or_default()
    });
    json_array.push(new_object);
    let updated_json = serde_json::to_string_pretty(&json_array)
        .map_err(|e| format!("Failed to serialize JSON: {}", e));
    fs::write(file_path, updated_json.unwrap())
        .map_err(|e| format!("Failed to write JSON back to file: {}", e))
        .unwrap();
    fs::read_to_string(file_path).expect("Failed to read existing file")
}

// 执行一个 start cmd命令
#[tauri::command]
fn start_cmd(cmd: String) -> String {
    let full_cmd = "start ".to_owned() + &cmd;
    println!("Executing command: {}", full_cmd);
    match Command::new("cmd").arg("/c").arg(full_cmd).output() {
        Ok(output) => {
            if output.status.success() {
                output.status.to_string()
            } else {
                format!("Error: Command failed with status code: {}", output.status)
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

#[derive(Deserialize, Debug, serde::Serialize)]
struct PositionSaver {
    x: i32,
    y: i32,
}

#[tauri::command]
fn save_position(window: Window) {
    let mut position: tauri::PhysicalPosition<i32> = window.outer_position().unwrap();
    if position.x < 0 {
        position.x = 0;
    }
    if position.y < 0 {
        position.y = 0;
    }
    let file_path = "./config.json";
    let _ = fs::write(
        file_path,
        serde_json::to_string(&PositionSaver {
            x: position.x,
            y: position.y,
        })
        .unwrap(),
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let file_path = "./config.json";
    let mut position_json = PositionSaver { x: 0, y: 0 };
    if !Path::new(file_path).exists() {
        // 选择创建一个默认的 JSON 文件
        let default_json = r#"{"x":0,"y":0}"#;
        File::create(file_path)
            .and_then(|mut f| f.write_all(default_json.as_bytes()))
            .expect("Failed to create default JSON file");
    } else {
        position_json = serde_json::from_str(&fs::read_to_string(&file_path).unwrap())
            .map_err(|e| format!("Failed to parse JSON string: {}", e))
            .unwrap();
    }
    tauri::Builder::default()
        .setup(move |app| {
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_position(tauri::PhysicalPosition::new(
                position_json.x,
                position_json.y,
            ));
            Ok(())
        })
        .manage(
            Arc::new(
                AppState {
                    current_song: Mutex::new(None)
                }
            )
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_json_data,
            start_cmd,
            upload_file_path,
            save_position,
            music_player::get_songs,
            music_player::play_song,
            music_player::pause_song,
            music_player::set_volume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
