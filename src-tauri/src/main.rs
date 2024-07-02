// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest::StatusCode;
use serde::Serialize;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tauri::Window;
use url::Url;

#[derive(Clone, Serialize)]
enum URLStatus {
    Downloading,
    Downloaded,
    Failed,
}

#[derive(Clone, serde::Serialize)]
struct URLPayload {
    url: String,
    status: URLStatus,
}

#[derive(Clone, serde::Serialize)]
struct BulkURLPayload {
    urls: Vec<URLPayload>,
}

#[derive(Clone, serde::Serialize)]
struct Message {
    message: String,
}

#[derive(Debug, Clone)]
struct NonOKResponseCode;

#[derive(Debug, Clone)]
struct FailedToWriteFile;

impl fmt::Display for NonOKResponseCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The download has failed because the remote server returned a non-OK (ex. 200) response code.")
    }
}
impl std::error::Error for NonOKResponseCode {}

impl fmt::Display for FailedToWriteFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The download has failed because the system has failed to save it."
        )
    }
}

impl std::error::Error for FailedToWriteFile {}

// TODO: use a crate?
// but i don't want too many dependencies..
#[allow(deprecated)]
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

// TODO: implement the errors above
async fn download_url(url: &str) -> Result<(), std::io::Error> {
    let parsed_url = Url::parse(url).unwrap();
    let path = Path::new(parsed_url.path());
    let file = path.file_name().unwrap().to_str().unwrap();

    let response = reqwest::get(url).await.unwrap();

    let mut file = File::create(get_downloads_folder() + file).unwrap();

    // rust error handling is a pain, rust </3
    match file.write(&response.bytes().await.unwrap().to_vec()) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[tauri::command]
// TODO: use it.. obviously
#[allow(unused_must_use)]
async fn download_urls(window: Window, file_path: &str) -> Result<(), ()> {
    println!("Attempting to read {}", file_path);
    let path = Path::new(file_path);
    if !path.exists() {
        window
            .emit(
                "new-url",
                Message {
                    message: String::from(
                        "Invalid path has been provided, and thus no files have been downloaded.",
                    ),
                },
            )
            .unwrap();
    }

    let contents = fs::read_to_string(file_path).unwrap();
    let urls = json::parse(contents.as_str()).unwrap();

    // whyy is this members?? and not values??
    // this makes no sense!
    let mut url_payloads: Vec<URLPayload> = Vec::new();

    for value in urls.members() {
        let url = value.as_str().unwrap();
        url_payloads.push(URLPayload {
            url: String::from(url),
            status: URLStatus::Downloading,
        })
    }

    window
        .emit("bulk-urls", BulkURLPayload { urls: url_payloads })
        .unwrap();

    // loop thru again to download
    for value in urls.members() {
        let url = value.as_str().unwrap();
        println!("Downloading {}..", url);

        // rust error handling is still a pain, and rust is still </3.
        match download_url(url).await {
            Ok(_) => {
                window
                    .emit(
                        "url-updated",
                        URLPayload {
                            url: String::from(url),
                            status: URLStatus::Downloaded,
                        },
                    )
                    .unwrap();
            }
            Err(_) => {
                window
                    .emit(
                        "url-updated",
                        URLPayload {
                            url: String::from(url),
                            status: URLStatus::Failed,
                        },
                    )
                    .unwrap();
            }
        }
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![download_urls])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
