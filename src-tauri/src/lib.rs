mod installers;
mod util;
mod kde_theme;

#[tauri::command]
fn get_kde_theme() -> Result<kde_theme::KdeTheme, String> {
    kde_theme::get_kde_theme()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            installers::flatpak::install_flatpak,
            installers::flatpak::uninstall_flatpak,
            get_kde_theme,
        ])
        .run(tauri::generate_context!())
        .expect("running tauri application");
}
