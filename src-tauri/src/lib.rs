mod cache;
mod installers;
mod util;
mod kde_theme;

macro_rules! simple_command {
    ($name:ident, $func:path, $ret:ty) => {
        #[tauri::command]
        async fn $name() -> Result<$ret, String> {
            $func().await
        }
    };
    ($name:ident, $func:path, $ret:ty, $($p:ident: $t:ty),+) => {
        #[tauri::command]
        async fn $name($($p: $t),+) -> Result<$ret, String> {
            $func($($p),+).await
        }
    };
}

macro_rules! simple_command_sync {
    ($name:ident, $func:path, $ret:ty) => {
        #[tauri::command]
        fn $name() -> Result<$ret, String> {
            $func()
        }
    };
    ($name:ident, $func:path, $ret:ty, $($p:ident: $t:ty),+) => {
        #[tauri::command]
        fn $name($($p: $t),+) -> Result<$ret, String> {
            $func($($p),+)
        }
    };
}


simple_command_sync!(get_kde_theme, kde_theme::get_kde_theme, kde_theme::KdeTheme);


simple_command_sync!(is_cache_ready_sync, cache::queries::is_cache_ready_sync, bool);
simple_command!(get_cached_apps_sync, cache::queries::get_cached_apps_sync, Vec<cache::CachedApp>);
simple_command!(get_cached_app_sync, cache::queries::get_cached_app_sync, Option<cache::CachedApp>, app_id: String);
simple_command!(get_cached_apps_batch_sync, cache::queries::get_cached_apps_batch_sync, Vec<cache::CachedApp>, app_ids: Vec<String>);
simple_command!(get_cached_categories_sync, cache::queries::get_cached_categories_sync, Vec<cache::CachedCategory>);
simple_command!(get_cached_category_collection_sync, cache::queries::get_cached_category_collection_sync, Option<cache::CachedCategoryCollection>, category_id: String);
simple_command!(get_cached_category_collection_with_apps_sync, cache::queries::get_category_with_apps_sync, Option<(cache::CachedCategoryCollection, Vec<cache::CachedApp>)>, category_id: String, limit: usize);
simple_command!(search_cached_apps_sync, cache::queries::search_cached_apps_sync, Vec<cache::SearchResult>, query: String);
simple_command!(get_cached_category_apps_paginated, cache::queries::get_category_apps_page_sync, (Vec<cache::CachedApp>, usize), category_id: String, limit: usize, offset: usize);
simple_command!(get_cached_collection_apps_sync, cache::queries::get_cached_collection_apps_sync, Vec<cache::CachedApp>, collection_type: String);
simple_command!(get_homepage_collections_sync, cache::queries::get_homepage_collections_sync, (Vec<cache::CachedApp>, Vec<cache::CachedApp>, Vec<cache::CachedApp>));
simple_command!(get_app_icons_batch_sync, cache::queries::get_app_icons_batch_sync, Vec<Option<String>>, app_ids: Vec<String>);

simple_command!(get_app_icon_data_url_sync, cache::queries::get_app_icon_data_url_sync, Option<String>, app_id: String);

#[tauri::command]
async fn initiate_cache(app: tauri::AppHandle, clear_cache: bool) -> Result<(), String> {
    cache::api::initiate_cache(app, clear_cache).await
}
simple_command!(download_and_cache_icon, cache::icons::download_and_cache_icon_command, Option<String>, app_id: String, icon_url: String);

#[tauri::command]
async fn fetch_and_cache_category_collection(app: tauri::AppHandle, category_id: String) -> Result<(), String> {
    let pool = cache::queries::get_db_pool().await.map_err(|e| e.to_string())?;
    cache::api::fetch_category_collection(&app, &pool, &category_id).await
}

#[tauri::command]
async fn fetch_and_cache_collection(app: tauri::AppHandle, collection_type: String) -> Result<Vec<String>, String> {
    let pool = cache::queries::get_db_pool().await.map_err(|e| e.to_string())?;
    cache::api::fetch_collection(&app, &pool, &collection_type).await
}

#[tauri::command]
async fn set_complete(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(splashscreen) = app.get_webview_window("splashscreen") {
        splashscreen
            .close()
            .map_err(|e| format!("Failed to close splashscreen: {}", e))?;
    }
    if let Some(main) = app.get_webview_window("main") {
        main.show()
            .map_err(|e| format!("Failed to show main window: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri_plugin_sql::{Migration, MigrationKind};
    
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_cache_tables",
            sql: include_str!("../migrations/001_create_cache_tables.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "add_icon_data_column",
            sql: include_str!("../migrations/002_add_icon_data_column.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "add_search_indexes",
            sql: include_str!("../migrations/003_add_search_indexes.sql"),
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            eprintln!("Tauri setup: Spawning background cache task...");
            tauri::async_runtime::spawn(async move {
                eprintln!("Background task: Waiting 500ms for database to be ready...");
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                eprintln!("Background task: Calling initiate_cache...");
                match cache::api::initiate_cache(app_handle, false).await {
                    Ok(_) => eprintln!("Background task: initiate_cache returned Ok"),
                    Err(e) => eprintln!("Background task: initiate_cache returned error: {}", e),
                }
            });
            eprintln!("Tauri setup: Background task spawned, returning Ok");
            Ok(())
        })
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:cache.db", migrations)
                .build()
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            installers::flatpak::install_flatpak,
            installers::flatpak::uninstall_flatpak,
            installers::flatpak::is_flatpak_installed,
            get_kde_theme,
            is_cache_ready_sync,
            get_cached_apps_sync,
            get_cached_app_sync,
            get_cached_apps_batch_sync,
            get_cached_categories_sync,
            get_cached_category_collection_sync,
            get_cached_category_collection_with_apps_sync,
            search_cached_apps_sync,
            get_cached_category_apps_paginated,
            get_cached_collection_apps_sync,
            get_homepage_collections_sync,
            get_app_icons_batch_sync,
            get_app_icon_data_url_sync,
            initiate_cache,
            download_and_cache_icon,
            fetch_and_cache_category_collection,
            fetch_and_cache_collection,
            set_complete
        ])
        .run(tauri::generate_context!())
        .expect("running tauri application");
}
