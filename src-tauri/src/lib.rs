mod guard;
mod models;
mod monitor;
mod scanner;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

use crate::models::{ActionResult, LiveSnapshot, ScanReport};

#[tauri::command]
async fn scan_apps(custom_roots: Option<Vec<String>>) -> Result<ScanReport, String> {
    tauri::async_runtime::spawn_blocking(move || scanner::scan_installed_apps(custom_roots))
        .await
        .map_err(|e| format!("スキャン実行エラー: {}", e))
}

#[tauri::command]
async fn get_live_snapshot() -> Result<LiveSnapshot, String> {
    tauri::async_runtime::spawn_blocking(monitor::collect_live_snapshot)
        .await
        .map_err(|e| format!("ライブ監視エラー: {}", e))
}

#[tauri::command]
async fn apply_env_opt_out(name: String, value: String) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || guard::set_user_env_var(&name, &value))
        .await
        .map_err(|e| format!("環境変数設定エラー: {}", e))
}

#[tauri::command]
async fn remove_env_opt_out(name: String) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || guard::remove_user_env_var(&name))
        .await
        .map_err(|e| format!("環境変数削除エラー: {}", e))
}

#[tauri::command]
async fn block_domains_in_hosts(domains: Vec<String>) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || guard::block_domains_in_hosts_elevated(domains))
        .await
        .map_err(|e| format!("hosts ブロックエラー: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show_i = MenuItem::with_id(app, "show", "Sentinel Guard を開く", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Sentinel Guard — Telemetry & Privacy Watch")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide to system tray instead of exiting
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            scan_apps,
            get_live_snapshot,
            apply_env_opt_out,
            remove_env_opt_out,
            block_domains_in_hosts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
