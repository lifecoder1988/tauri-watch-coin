// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::to_string;
use tauri::App;
use tauri::{
    AppHandle, CustomMenuItem, Icon, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
};

use rusttype::{Font, Scale};

use image::{ImageBuffer, Rgba};
use tauri_plugin_store::StoreBuilder;
use tauri_plugin_store::{Store, StoreCollection};

use std::thread;

use tauri::api::path::data_dir;

use std::fs;

use std::path::Path;

use reqwest::Error;
use serde::{Deserialize, Deserializer};

use std::path::PathBuf;

use tauri::SystemTrayHandle;
use tauri::Wry;
use tauri_plugin_store::with_store;
use tokio::runtime::Runtime;
use tokio::time::{self, Duration};

use std::sync::{Arc, Mutex};
use tauri::State;

const BUNDLE_IDENTIFIER: &str = "com.moyu.kline";

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 实现从字符串到 f64 的转换
fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    bitcoin: Option<BtcData>,
}

#[derive(Deserialize, Debug)]
struct BtcData {
    usd: f32,
}

#[derive(Deserialize, Debug)]
struct Ticker {
    #[serde(deserialize_with = "string_to_f64")]
    last: f64, // 你可以根据API返回的内容调整字段

    #[serde(rename = "percentChange", deserialize_with = "string_to_f64")]
    percent_change: f64,
}

fn format_price(pair: &str, price: f64) -> String {
    let parts: Vec<&str> = pair.split('/').collect();

    format!("{} {:>10}", parts[0], format!("${}", price))
}

fn format_percent(percent: f64) -> String {
    if percent < 0.0 {
        return format!("{:.2}%", percent);
    }
    return format!("+{:.2}%", percent);
}

fn is_negetive(percent: &str) -> bool {
    let first_char = percent.chars().next();
    match first_char {
        Some(c) => {
            if c == '-' {
                return true;
            } else {
                return false;
            }
        }
        None => {
            return false;
        }
    }
}
async fn get_coin_price(pair: &str) -> Result<Ticker, Error> {
    let lower_pair = pair.to_lowercase().replace("/", "_");

    let url = format!("https://api.gate.io/api2/1/ticker/{}", lower_pair);

    println!("{}", url);
    // 发送GET请求
    let response = reqwest::get(url).await?;

    // 解析JSON响应
    let ticker: Ticker = response.json().await?;

    println!("BOME 最新价格: {:?}", ticker);

    Ok(ticker)
}

fn get_app_data_path() -> Option<PathBuf> {
    if let Some(data_dir) = data_dir() {
        // 使用你的应用的 Bundle Identifier
        let app_data_path = data_dir.join(BUNDLE_IDENTIFIER);
        Some(app_data_path)
    } else {
        None
    }
}

fn generate_icon(pair: &str, text: &str, percent: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new image buffer with RGBA values.
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(1000, 60);

    // Load the font
    let font_data = include_bytes!("../fonts/Roboto-Bold.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");

    // Specify the scale and positioning of the text.
    let scale = Scale { x: 54.0, y: 54.0 };
    //let text = "Hi";
    let spacing = 5.0; // 字符间的额外间距

    // Iterate over the characters in the text, drawing each one onto the image.

    let mut cursor_x = 10.0;

    for (i, c) in text.chars().enumerate() {
        println!("{}", c);
        let glyph = font
            .glyph(c)
            .scaled(scale)
            .positioned(rusttype::point(cursor_x, 44.0));

        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                //println!("{:?}", bounding_box);
                let x = x + bounding_box.min.x as u32;
                let y = y + bounding_box.min.y as u32;
                // `v` is a value between 0.0 and 1.0 representing the glyph coverage for the pixel.
                // Here, it's used to determine the pixel color.
                let pixel = image.get_pixel_mut(x, y);
                *pixel = Rgba([0, 0, 0, (v * 255.0) as u8]);
            });
        }
        cursor_x += glyph.unpositioned().h_metrics().advance_width + spacing;
        println!("{}", cursor_x);
    }

    //let text2 = "+1.5%(1d)";
    let mut font_color = Rgba([22, 199, 132, 255]);

    if is_negetive(percent) {
        //rgb(22, 199, 132)
        font_color = Rgba([255, 0, 0, 255]);
    }
    let scale = Scale { x: 40.0, y: 40.0 };

    cursor_x += 30.0;

    for (i, c) in percent.chars().enumerate() {
        let glyph = font
            .glyph(c)
            .scaled(scale)
            .positioned(rusttype::point(cursor_x, 40.0));

        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                //println!("{:?}", bounding_box);
                let x = x + bounding_box.min.x as u32;
                let y = y + bounding_box.min.y as u32;
                // `v` is a value between 0.0 and 1.0 representing the glyph coverage for the pixel.
                // Here, it's used to determine the pixel color.
                let pixel = image.get_pixel_mut(x, y);

                *pixel = Rgba([
                    font_color[0],
                    font_color[1],
                    font_color[2],
                    (v * 255.0) as u8,
                ]);
            });
        }
        cursor_x += glyph.unpositioned().h_metrics().advance_width + spacing;
    }

    println!("image save");

    /*let relative_path = Path::new("../public/icon1.png");
    match fs::canonicalize(relative_path) {
        Ok(absolute_path) => println!("完整路径: {}", absolute_path.display()),
        Err(e) => println!("错误获取完整路径: {}", e),
    }*/
    if let Some(mut path) = get_app_data_path() {
        path.push("icon1.png");
        image.save(path.as_path()).unwrap();
    }

    Ok(())
}

async fn render(
    app_handle: &AppHandle,
    tray_handle: &SystemTrayHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let stores = app_handle.state::<StoreCollection<Wry>>();
    let path = PathBuf::from(".store.json");

    let result = with_store(app_handle.clone(), stores, path, |store| {
        //store
        //let r = store.get("pair");

        let r = store
            .get("pair")
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "12"))?;

        if let Some(val) = r.as_str() {
            return Ok(val.to_string());
        }

        return Ok("BTC/USDT".to_string());
    });
    let pair = result?;
    println!("{}", pair);

    let ticker = get_coin_price(&pair).await?;
    let _ = generate_icon(
        &pair,
        &format_price(&pair, ticker.last),
        &format_percent(ticker.percent_change),
    );

    let mut path = get_app_data_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "13"))?;
    path.push("icon1.png");
    let icon_path = path.as_path();
    match fs::read(icon_path) {
        Ok(bytes) => {
            // 成功读取字节后的操作
            //set_tray_icon(bytes);
            //println!("{:?}", bytes,);

            tray_handle.set_icon(tauri::Icon::Raw(bytes)).unwrap();
        }
        Err(e) => {
            eprintln!("读取图标文件失败: {}", e);
            // 错误处理代码
        }
    }

    Ok(())
}

#[derive(Debug)]
struct MySetting {
    pair: String,
}

#[tauri::command]
fn get_setting(
    app_handle: tauri::AppHandle,
    stores: tauri::State<StoreCollection<Wry>>,
    key: &str,
) -> Result<String, tauri_plugin_store::Error> {
    let path = PathBuf::from(".store.json");

    //let result = with_store(app_handle, stores, path, |store| store.insert("a".to_string(), json!("b")));

    let result = with_store(app_handle, stores, path, |store| {
        //store
        let r = store.get(key);
        println!("{:?}", r);
        if let Some(val) = r {
            return Ok(val.as_str().unwrap().to_owned());
        }
        return Ok("".to_string());
    });
    return result;
}

#[tauri::command]
fn set_setting(
    app_handle: tauri::AppHandle,
    stores: tauri::State<StoreCollection<Wry>>,
    key: &str,
    value: &str,
) -> Result<(), tauri_plugin_store::Error> {
    let path = PathBuf::from(".store.json");

    //let result = with_store(app_handle, stores, path, |store| store.insert("a".to_string(), json!("b")));

    let result = with_store(app_handle, stores, path, |store| {
        //store
        store.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        store.save()
    });
    return result;
}

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Show App"))
        .add_item(CustomMenuItem::new("quit", "Quit"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![greet, get_setting, set_setting])
        .setup(|app| {
            let tray_handle = app.tray_handle();

            // 在另一个线程中周期性更新系统托盘图标

            let app_handle = app.handle();
            let stores = app_handle.state::<StoreCollection<Wry>>();

            thread::spawn(move || {
                // 在这个新线程中创建并启动一个新的 Tokio 运行时
                let rt = Runtime::new().unwrap();

                // 使用运行时的 `block_on` 来运行异步代码块
                rt.block_on(async {
                    // 创建一个每5秒触发一次的定时器
                    let mut interval = time::interval(Duration::from_secs(3));

                    loop {
                        interval.tick().await; // 等待下一个定时器触发

                        match render(&app_handle, &tray_handle).await {
                            Ok(()) => {
                                println!("render success");
                            }
                            Err(e) => {
                                println!("{:?}", e);
                            }
                        }
                    }
                });
            });

            Ok(())
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
