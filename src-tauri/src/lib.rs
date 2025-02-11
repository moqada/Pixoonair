// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Deserialize;
use settings::AppSettings;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{ActivationPolicy, RunEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log;
use tauri_plugin_store::StoreExt;

mod camera;
mod pixoo;
mod settings;
mod tray;

const SETTINGS_FILE_NAME: &str = "settings.json";
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum DisplayMode {
    Normal,
    OnAir,
}

#[tauri::command]
async fn save_settings(
    app_handle: tauri::AppHandle,
    target_device_name: &str,
    gif_file_id: &str,
    gif_file_url: &str,
    gif_file_type: settings::GifFileType,
) -> Result<(), String> {
    log::debug!("call save_settings");
    let res = app_handle.store(SETTINGS_FILE_NAME.to_string());
    match res {
        Ok(_) => log::debug!("store opened"),
        Err(e) => {
            return Err(format!("store open error: {}", e.to_string()));
        }
    };
    let store = res.unwrap();
    let app_settings = AppSettings {
        target_device_name: target_device_name.to_string(),
        gif_file_id: gif_file_id.to_string(),
        gif_file_url: gif_file_url.to_string(),
        gif_file_type: gif_file_type,
    };
    match app_settings.save(&store) {
        Ok(_) => log::debug!("settings saved"),
        Err(e) => {
            return Err(format!("settings save error: {}", e.to_string()));
        }
    }
    Ok(())
}

#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<AppSettings, String> {
    log::debug!("call load_settings");
    let res = app_handle.store(SETTINGS_FILE_NAME.to_string());
    match res {
        Ok(_) => log::debug!("store opened"),
        Err(e) => {
            return Err(format!("store open error: {}", e.to_string()));
        }
    };
    let store = res.unwrap();
    let app_settings = settings::AppSettings::load_from_store(&store);
    Ok(app_settings)
}

#[tauri::command]
async fn change_display_mode(
    app_handle: tauri::AppHandle,
    mode: DisplayMode,
) -> Result<(), String> {
    let settings = load_settings(app_handle)?;
    match mode {
        DisplayMode::Normal => activate_normal_mode(settings).await?,
        DisplayMode::OnAir => activate_on_air_mode(settings).await?,
    }
    Ok(())
}

async fn get_device_by_name(device_name: &str) -> Result<Option<pixoo::Device>, String> {
    let devices = pixoo::get_devices().await.map_err(|e| e.to_string())?;
    Ok(devices.into_iter().find(|d| d.device_name == device_name))
}

async fn activate_normal_mode(app_settings: AppSettings) -> Result<(), String> {
    log::debug!("call activate_normal_mode");
    let device = match get_device_by_name(&app_settings.target_device_name).await? {
        Some(device) => device,
        None => return Ok(()),
    };
    log::info!(
        "device found: ID: {}, Name: {}, IP: {}, MAC: {}, HARDWARE: {}",
        device.device_id,
        device.device_name,
        device.device_private_i_p,
        device.device_mac,
        device.hardware
    );
    let channel_id = pixoo::get_current_channel(&device.device_private_i_p).await?;
    if channel_id == pixoo::ChannelId::Unknown {
        return Err(format!("Channel ID is unknown: {:?}", channel_id));
    }
    pixoo::set_current_channel(&device.device_private_i_p, channel_id).await?;
    Ok(())
}

async fn activate_on_air_mode(app_settings: AppSettings) -> Result<(), String> {
    log::debug!("call activate_on_air_mode");
    let device = match get_device_by_name(&app_settings.target_device_name).await? {
        Some(device) => device,
        None => return Ok(()),
    };
    log::info!(
        "device found: ID: {}, Name: {}, IP: {}, MAC: {}, HARDWARE: {}",
        device.device_id,
        device.device_name,
        device.device_private_i_p,
        device.device_mac,
        device.hardware
    );
    if app_settings.gif_file_type == settings::GifFileType::Id && app_settings.gif_file_id != "" {
        pixoo::play_divoom_gif(&device.device_private_i_p, app_settings.gif_file_id).await?;
    } else if app_settings.gif_file_type == settings::GifFileType::Url
        && app_settings.gif_file_url != ""
    {
        pixoo::play_gif(&device.device_private_i_p, &app_settings.gif_file_url).await?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_on_exit = Arc::clone(&stop_flag);
    let main_app = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            app.handle().plugin(tauri_plugin_positioner::init())?;
            tray::init(app)?;
            app.set_activation_policy(ActivationPolicy::Accessory); // Dock アイコンを表示しない
            let store = app.store(SETTINGS_FILE_NAME.to_string())?;
            settings::AppSettings::load_from_store(&store);
            camera::start_monitoring(
                {
                    let store_clone = Arc::clone(&store);
                    move || {
                        let app_settings = settings::AppSettings::load_from_store(&store_clone);
                        tauri::async_runtime::spawn(activate_on_air_mode(app_settings));
                    }
                },
                {
                    let store_clone = Arc::clone(&store);
                    move || {
                        let app_settings = settings::AppSettings::load_from_store(&store_clone);
                        tauri::async_runtime::spawn(activate_normal_mode(app_settings));
                    }
                },
                stop_flag,
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            change_display_mode,
            load_settings,
            save_settings
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    main_app.run(move |_app_handle, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            if stop_flag_on_exit.load(Ordering::Relaxed) {
                return;
            }
            stop_flag_on_exit.store(true, Ordering::Relaxed);
            api.prevent_exit();
            // 別スレッドで Cleanup 処理
            let app_handle = _app_handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(1)); // 停止処理を待つ
                log::info!("Exiting app...");
                app_handle.exit(0);
            });
        }
        _ => {}
    });
}
