use serde::Serialize;
use std::fs::{self, File};
use std::sync::Arc;
use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::thread;
use tauri::{AppHandle, Emitter, State};

use crate::AppState;

#[derive(Serialize)]
pub struct Song {
    pub label: String,
    pub value: String,
}

#[tauri::command]
pub fn get_songs() -> Vec<Song> {
    let mut music_files = Vec::new();

    let entries = fs::read_dir("../assets/mp3").unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if file_name_str.ends_with(".mp3") {
                        let song = Song {
                            label: file_name_str.to_string(),
                            value: file_name_str.to_string(),
                        };
                        music_files.push(song);
                    }
                }
            }
        }
    }
    music_files
}

#[tauri::command]
pub fn play_song(app:AppHandle, label: String,vol: f32, state: State<'_, Arc<AppState>>) {
    let path = format!("../assets/mp3/{}", label);
    let state = state.inner().clone();

    thread::spawn(move || {
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error opening file {} : {}", path, e);
                return;
            }
        };

        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Error initializing output stream : {}", e);
                return;
            }
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => Arc::new(sink),
            Err(e) => {
                eprintln!("Error playing : {}", e);
                return;
            }
        };
        match Decoder::new(BufReader::new(file)) {
            Ok(source) => sink.append(source),
            Err(e) => {
                eprintln!("Error decoding : {}; Song info : {}", e, label);
                return;
            }
        }

        {
            let mut current_song = state.current_song.lock().unwrap();
            if let Some(ref current) = *current_song {
                current.pause();
            }

            *current_song = Some(sink.clone());
        }

        sink.set_volume(vol);
        sink.sleep_until_end();
        app.emit("play_finish", true).unwrap()
    });
}

#[tauri::command]
pub fn pause_song(state: State<'_, Arc<AppState>>) {
    let current_song = state.current_song.lock().unwrap();
    if let Some(ref sink) = *current_song {
        sink.pause()
    }
}

#[tauri::command]
pub fn set_volume(vol: f32, state: State<'_, Arc<AppState>>) {
    let current_song = state.current_song.lock().unwrap();
    if let Some(ref sink) = *current_song {
        sink.set_volume(vol);
    }
}