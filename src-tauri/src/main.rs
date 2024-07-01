// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::fs::{self, File};
use std::io::{Error, Write};
use std::path::Path;
use url::{ParseError, Url};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn get_downloads_folder() -> String {
    // TODO: implement support for more than just windows
    if cfg!(windows) {
        return std::env::home_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            .to_string()
            + "/Downloads/";
    } else {
        return String::from("no-home-folder-boop");
    }
}

async fn download_url(url: &str) {
    let parsed_url = Url::parse(url).unwrap();
    let path = Path::new(parsed_url.path());
    let file = path.file_name().unwrap().to_str().unwrap();

    let response = reqwest::get(url).await.unwrap();

    let mut file = File::create(get_downloads_folder() + file).unwrap();
    file.write(&response.bytes().await.unwrap().to_vec());
}

#[tauri::command]
async fn download_urls(file_path: &str) -> Result<String, String> {
    println!("Attempting to read {}", file_path);
    let path = Path::new(file_path);
    if !path.exists() {
        String::from("Invalid path has been provided, and thus no files have been downloaded.");
    }

    let contents = fs::read_to_string(file_path).unwrap();
    let urls = json::parse(contents.as_str()).unwrap();

    for value in urls.members() {
        let url = value.as_str().unwrap();
        println!("Value: {}", url);
        download_url(url).await;
    }

    Ok(String::from("meow"))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, download_urls])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
