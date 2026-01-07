use std::env;
use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    Emitter,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // --- MENU SETUP ---
            let handle = app.handle();

            // 1. Valter Menu (App Name)
            let rescan_item =
                MenuItem::with_id(handle, "rescan", "Rescan System", true, Some("CmdOrCtrl+R"))?;
            let config_item = MenuItem::with_id(
                handle,
                "open_config",
                "Open Configuration",
                true,
                Some("CmdOrCtrl+,"),
            )?;

            let app_menu = Submenu::with_items(
                handle,
                "Valter",
                true,
                &[
                    &rescan_item,
                    &config_item,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, Some("Quit"))?,
                ],
            )?;

            // 2. Standard Menus
            let edit_menu = Submenu::with_items(
                handle,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(handle, None)?,
                    &PredefinedMenuItem::redo(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, None)?,
                    &PredefinedMenuItem::copy(handle, None)?,
                    &PredefinedMenuItem::paste(handle, None)?,
                    &PredefinedMenuItem::select_all(handle, None)?,
                ],
            )?;

            let view_menu = Submenu::with_items(
                handle,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(handle, None)?],
            )?;

            let window_menu = Submenu::with_items(
                handle,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::close_window(handle, None)?,
                ],
            )?;

            // BUILD MENU
            let menu =
                Menu::with_items(handle, &[&app_menu, &edit_menu, &view_menu, &window_menu])?;

            app.set_menu(menu)?;

            // MENU EVENTS
            app.on_menu_event(move |app_handle, event| {
                if event.id() == rescan_item.id() {
                    println!("Menu: Rescan triggered");
                    let _ = app_handle.emit("menu-rescan", ());
                } else if event.id() == config_item.id() {
                    println!("Menu: Open Config triggered");
                    // Resolve config path again (a bit repetitive, but safe)
                    let is_dev = cfg!(debug_assertions);
                    let valter_home = if let Ok(h) = env::var("VALTER_HOME") {
                        PathBuf::from(h)
                    } else if is_dev {
                        // Heuristic: try to find root
                        let p = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        if p.join("valter.dev.config").exists() {
                            p
                        } else if p
                            .parent()
                            .map(|x| x.join("valter.dev.config").exists())
                            .unwrap_or(false)
                        {
                            p.parent().unwrap().to_path_buf()
                        } else if p
                            .parent()
                            .and_then(|x| x.parent())
                            .map(|x| x.join("valter.dev.config").exists())
                            .unwrap_or(false)
                        {
                            p.parent().unwrap().parent().unwrap().to_path_buf()
                        } else {
                            PathBuf::from("../../")
                        }
                    } else {
                        env::var("HOME")
                            .or_else(|_| env::var("USERPROFILE"))
                            .map(|h| PathBuf::from(h).join(".valter"))
                            .unwrap_or_else(|_| PathBuf::from(".valter"))
                    };

                    let config_file = if is_dev {
                        valter_home.join("valter.dev.config")
                    } else {
                        valter_home.join("valter.config")
                    };

                    if let Err(e) = open::that(&config_file) {
                        eprintln!("Failed to open config: {}", e);
                    }
                }
            });

            // --- CORE STARTUP ---
            // Initialize tracing for backend logs
            let _ = tracing_subscriber::fmt().try_init();

            // Spawn Valter Core
            tauri::async_runtime::spawn(async move {
                let is_dev = cfg!(debug_assertions);

                let valter_home = if let Ok(h) = env::var("VALTER_HOME") {
                    PathBuf::from(h)
                } else if is_dev {
                    let path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    if path.join("valter.dev.config").exists() {
                        path
                    } else if path
                        .parent()
                        .map(|p| p.join("valter.dev.config").exists())
                        .unwrap_or(false)
                    {
                        path.parent().unwrap().to_path_buf()
                    } else if path
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.join("valter.dev.config").exists())
                        .unwrap_or(false)
                    {
                        path.parent().unwrap().parent().unwrap().to_path_buf()
                    } else {
                        PathBuf::from("../../")
                    }
                } else {
                    env::var("HOME")
                        .or_else(|_| env::var("USERPROFILE"))
                        .map(|h| PathBuf::from(h).join(".valter"))
                        .unwrap_or_else(|_| PathBuf::from(".valter"))
                };

                let abs_home = std::fs::canonicalize(&valter_home).unwrap_or(valter_home.clone());
                println!("🚀 Starting Valter Core at {:?}", abs_home);

                if let Err(e) = valter_core::run(abs_home, is_dev).await {
                    eprintln!("Valter Core Error: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
