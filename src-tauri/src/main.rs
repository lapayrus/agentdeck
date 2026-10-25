#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::AppState;
use std::sync::Mutex;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::load())
        .manage(commands::WatcherState(Mutex::new(None)))
        .setup(|app| {
            *app.state::<commands::WatcherState>()
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner()) =
                commands::start_watcher(app.handle().clone(), None);
            let window_state = {
                let state = app.state::<AppState>();
                let settings = state
                    .settings
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                settings.window.clone()
            };
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(PhysicalSize::new(
                    window_state.width.max(800),
                    window_state.height.max(600),
                ));
                if let (Some(x), Some(y)) = (window_state.x, window_state.y) {
                    let visible = window
                        .available_monitors()
                        .map(|monitors| {
                            monitors.iter().any(|monitor| {
                                let position = monitor.position();
                                let size = monitor.size();
                                x + 80 >= position.x
                                    && y + 80 >= position.y
                                    && x < position.x + size.width as i32
                                    && y < position.y + size.height as i32
                            })
                        })
                        .unwrap_or(false);
                    if visible {
                        let _ = window.set_position(PhysicalPosition::new(x, y));
                    }
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                let (Ok(position), Ok(size)) = (window.outer_position(), window.inner_size())
                else {
                    return;
                };
                if size.width == 0
                    || size.height == 0
                    || position.x <= -30000
                    || position.y <= -30000
                {
                    return;
                }
                let state = window.state::<AppState>();
                let mut settings = state
                    .settings
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                settings.window.x = Some(position.x);
                settings.window.y = Some(position.y);
                settings.window.width = size.width;
                settings.window.height = size.height;
                let _ = state.store.save(&settings);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan,
            commands::set_project,
            commands::get_settings,
            commands::save_settings,
            commands::read_file,
            commands::plan_toggle,
            commands::plan_remove,
            commands::apply_plan,
            commands::restore,
            commands::list_trash,
            commands::list_history,
            commands::read_snapshot,
            commands::diff_snapshot,
            commands::rollback,
            commands::prune_snapshots,
            commands::history_for_artifact
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgentDeck");
}
