mod cloudflared;
mod commands;
mod error;
mod state;
mod types;

use std::time::Duration;

use sysinfo::{ProcessesToUpdate, System};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_notification::NotificationExt;

use crate::commands::prefs::{load_prefs_sync, PrefsState};
use crate::state::RuntimeState;

#[tauri::command]
fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

const ORPHAN_MIN_AGE_SECS: u64 = 30;

fn kill_orphan_instances() {
    let me = std::process::id();
    let our_exe = std::env::current_exe().ok();
    let our_exe_name = our_exe
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "flaredeck.exe".to_string());

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut killed_any = false;
    for (pid, process) in system.processes() {
        let pid_u32: u32 = (*pid).as_u32();
        if pid_u32 == me {
            continue;
        }
        let name = process.name().to_string_lossy().to_string();
        if !name.eq_ignore_ascii_case(&our_exe_name) && !name.eq_ignore_ascii_case("flaredeck.exe")
        {
            continue;
        }
        let age_secs = now_secs.saturating_sub(process.start_time());
        if age_secs < ORPHAN_MIN_AGE_SECS {
            continue;
        }
        eprintln!("killing orphan flaredeck process pid={pid_u32} age={age_secs}s");
        process.kill();
        killed_any = true;
    }

    if killed_any {
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    kill_orphan_instances();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(RuntimeState::new())
        .manage(PrefsState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let prefs = load_prefs_sync(&handle);
            if let Some(state) = handle.try_state::<PrefsState>() {
                if let Ok(mut guard) = state.prefs.lock() {
                    *guard = prefs;
                }
            }

            let show = MenuItem::with_id(&handle, "show", "Show FlareDeck", true, None::<&str>)?;
            let quit = MenuItem::with_id(&handle, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(&handle, &[&show, &quit])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(handle.default_window_icon().cloned().ok_or_else(|| {
                    tauri::Error::AssetNotFound("default window icon".to_string())
                })?)
                .tooltip("FlareDeck")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(&handle)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" {
                    return;
                }
                let app = window.app_handle();
                let prefs = app
                    .try_state::<PrefsState>()
                    .and_then(|s| s.prefs.lock().ok().map(|p| p.clone()))
                    .unwrap_or_default();

                if !prefs.close_choice_made {
                    api.prevent_close();
                    let _ = app.emit("window:first-close-prompt", ());
                    show_main_window(app);
                    return;
                }

                if prefs.minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                    if !prefs.tray_hint_shown {
                        let _ = app.emit("window:hidden-to-tray", ());
                        if let Err(e) = app
                            .notification()
                            .builder()
                            .title("FlareDeck is still running")
                            .body("Closed window hides FlareDeck to the system tray. Right-click the tray icon to bring it back or quit.")
                            .show()
                        {
                            eprintln!("notification failed: {e}");
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            commands::tunnel::cloudflared_check,
            commands::tunnel::tunnel_status,
            commands::tunnel::tunnel_start,
            commands::tunnel::tunnel_stop,
            commands::tunnel::tunnel_restart,
            commands::tunnel::tunnel_list,
            commands::tunnel::tunnel_route_dns,
            commands::tunnel::tunnel_create,
            commands::auth::auth_check,
            commands::auth::auth_login,
            commands::auth::auth_logout,
            commands::config::config_get,
            commands::config::config_save,
            commands::network::network_check_port,
            commands::dns::dns_check,
            commands::shell::shell_open_external,
            commands::shell::shell_open_path,
            commands::profiles::profiles_list,
            commands::profiles::profiles_create,
            commands::profiles::profiles_update,
            commands::profiles::profiles_delete,
            commands::profiles::profiles_set_active,
            commands::wsl::wsl_host_ip,
            commands::prefs::prefs_get,
            commands::prefs::prefs_set_minimize_to_tray,
            commands::prefs::prefs_mark_tray_hint_shown,
            commands::prefs::prefs_set_close_choice,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FlareDeck");
}
