use std::env;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|_app| {
      // Initialize tracing for backend logs
      // We ignore the result in case it's already initialized (though it shouldn't be)
      let _ = tracing_subscriber::fmt().try_init();

      // Spawn Valter Core
      tauri::async_runtime::spawn(async move {
          let is_dev = cfg!(debug_assertions);
          
          let valter_home = if let Ok(h) = env::var("VALTER_HOME") {
              PathBuf::from(h)
          } else if is_dev {
             // Try to find the root of the monorepo
             let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
             // If we are in app/src-tauri, root is ../..
             // If we are in app, root is ..
             // Check if valter.dev.config exists in current, or parent, or parent's parent.
             
             if path.join("valter.dev.config").exists() {
                 path
             } else if path.parent().map(|p| p.join("valter.dev.config").exists()).unwrap_or(false) {
                 path.parent().unwrap().to_path_buf()
             } else if path.parent().and_then(|p| p.parent()).map(|p| p.join("valter.dev.config").exists()).unwrap_or(false) {
                 path.parent().unwrap().parent().unwrap().to_path_buf()
             } else {
                 // Fallback to assuming we are in app/src-tauri and need to go up two levels
                 // This is a rough heuristic.
                 PathBuf::from("../../") 
             }
          } else {
              // Prod: ~/.valter
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