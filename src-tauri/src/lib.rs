mod installers;
mod util;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            installers::flatpak::install_flatpak,
            installers::flatpak::uninstall_flatpak
        ])
        .run(tauri::generate_context!())
        .expect("running tauri application");
}
