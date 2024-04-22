// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{to_string, Value};

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
use serde::{Deserialize, Deserializer, Serialize};

use std::path::PathBuf;

use tauri::SystemTrayHandle;
use tauri::Wry;
use tauri_plugin_store::with_store;
use tokio::runtime::Runtime;
use tokio::time::{self, Duration};

use std::sync::{Arc, Mutex};
use tauri::State;

use reqwest::header::{HeaderMap, REFERER};
use reqwest::Client;

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

fn format_price2(pair: &str, price: f64) -> String {
    let mut name = "";
    if pair == "sh000001".to_string() {
        name = "上证指数";
    } else if pair == "sh000300".to_string() {
        name = "沪深300";
    } else if pair == "sh601012".to_string() {
        name = "隆基绿能";
    }
    format!("{} {:>10}", name, format!("¥{:.2}", price))
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
async fn get_coin_price(pair: &str) -> Result<Ticker, Box<dyn std::error::Error>> {
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

async fn get_china_price(pair: &str) -> Result<Ticker, Box<dyn std::error::Error>> {
    /*let url = format!(
        "https://m.joudou.com/p/www/stockinfogate/indexqt/realtimeinfo?indexids={}",
        pair
    );*/

    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Referer", "https://finance.sina.com.cn".parse()?);

    let request = client
        .request(
            reqwest::Method::GET,
            format!("https://hq.sinajs.cn/list={}", pair),
        )
        .headers(headers);

    let response = request.send().await?;
    let data = response.text().await?;

    let parts: Vec<_> = data.split("=").collect();
    let s = parts[1].to_string();
    let s2 = s.replace("\"", "");
    let parts2: Vec<_> = s2.split(",").collect();
    println!("{:?}", parts2);
    let last: f64 = parts2[3].parse().unwrap();
    let yestorday: f64 = parts2[2].parse().unwrap();
    return Ok(Ticker {
        last: last,
        percent_change: (last - yestorday) * 100.0 / yestorday,
    });
    // 解析JSON响应
    /*let v: Value = response.json().await?;
    if let Some(data_array) = v["data"].as_array() {
        return Ok(Ticker {
            last: data_array[0]["newprice"].as_f64().unwrap(),
            percent_change: data_array[0]["changeratio"].as_f64().unwrap(),
        });
    }*/

    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        "12",
    )))
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

fn generate_icon(
    market_type: &str,
    pair: &str,
    text: &str,
    percent: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new image buffer with RGBA values.
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(1000, 60);

    // Load the font
    let font_data = include_bytes!("../fonts/Noto_Sans_SC/static/NotoSansSC-Bold.ttf");
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

    const BLUE_COLOR: image::Rgba<u8> = Rgba([22, 199, 132, 255]);
    const RED_COLOR: image::Rgba<u8> = Rgba([255, 0, 0, 255]);
    let mut font_color;

    if market_type == "crypto" {
        if is_negetive(percent) {
            font_color = RED_COLOR;
        } else {
            font_color = BLUE_COLOR;
        }
    } else {
        if is_negetive(percent) {
            font_color = BLUE_COLOR;
        } else {
            font_color = RED_COLOR;
        }
    }
    //let text2 = "+1.5%(1d)";
    //let mut font_color = Rgba([22, 199, 132, 255]);

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

#[derive(Serialize, Deserialize, Debug)]
struct JsonConfig {
    #[serde(rename = "type")]
    type_field: String,
    value: String,
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
            .get("config")
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "12"))?;

        if let Some(val) = r.as_str() {
            let cfg: JsonConfig = serde_json::from_str(val)?;
            return Ok(cfg);
        }

        return Ok(JsonConfig {
            type_field: "crypto".to_string(),
            value: "BTC/USDT".to_string(),
        });
    });

    println!("{:?}", result);

    let config = result?;
    let ticker;
    if config.type_field == "crypto".to_string() {
        ticker = get_coin_price(&config.value).await?;
        let _ = generate_icon(
            &config.type_field,
            &config.value,
            &format_price(&config.value, ticker.last),
            &format_percent(ticker.percent_change),
        );
    } else {
        ticker = get_china_price(&config.value).await?;
        let _ = generate_icon(
            &config.type_field,
            &config.value,
            &format_price2(&config.value, ticker.last),
            &format_percent(ticker.percent_change),
        );
    }

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
