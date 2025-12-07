use crate::cache::types::*;
use sqlx::sqlite::{SqlitePool, SqliteRow, SqliteConnectOptions};
use sqlx::Row;
use std::sync::OnceLock;
use std::str::FromStr;
use directories::ProjectDirs;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

static DB_POOL: OnceLock<tokio::sync::Mutex<Option<SqlitePool>>> = OnceLock::new();

pub async fn get_db_pool() -> Result<SqlitePool, String> {
    let pool_mutex = DB_POOL.get_or_init(|| tokio::sync::Mutex::new(None));
    let mut pool_guard = pool_mutex.lock().await;
    
    if let Some(ref pool) = *pool_guard {
        return Ok(pool.clone());
    }
    
    let project_dirs = ProjectDirs::from("", "", "softwarehub")
        .ok_or("Failed to get project directories")?;
    
    let db_path = project_dirs.data_dir().join("cache.db");
    
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    
    let mut opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))
        .map_err(|e| format!("Failed to parse database URL: {}", e))?;
    opts = opts.create_if_missing(true);
    
    let pool = SqlitePool::connect_with(opts)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    let tables_exist = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='apps'")
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("Failed to check if tables exist: {}", e))?;
    
    if tables_exist.is_none() {
        sqlx::query("CREATE TABLE IF NOT EXISTS apps (app_id TEXT PRIMARY KEY, name TEXT, description TEXT, summary TEXT, download_flatpak_ref TEXT, icon_url TEXT, icon_path TEXT, cached_at INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create apps table: {}", e))?;
        
        sqlx::query("CREATE TABLE IF NOT EXISTS categories (id TEXT PRIMARY KEY, name TEXT NOT NULL, cached_at INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create categories table: {}", e))?;
        
        sqlx::query("CREATE TABLE IF NOT EXISTS category_collections (category_id TEXT PRIMARY KEY, total_hits INTEGER NOT NULL, cached_at INTEGER NOT NULL)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create category_collections table: {}", e))?;
        
        sqlx::query("CREATE TABLE IF NOT EXISTS category_collection_apps (category_id TEXT NOT NULL, app_id TEXT NOT NULL, position INTEGER NOT NULL, PRIMARY KEY (category_id, app_id), FOREIGN KEY (category_id) REFERENCES category_collections(category_id) ON DELETE CASCADE)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create category_collection_apps table: {}", e))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_apps_cached_at ON apps(cached_at)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create index: {}", e))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_category_collection_apps_category ON category_collection_apps(category_id)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create index: {}", e))?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_category_collection_apps_position ON category_collection_apps(category_id, position)")
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to create index: {}", e))?;
        
        let column_exists = sqlx::query("SELECT name FROM pragma_table_info('apps') WHERE name='icon_data'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("Failed to check if icon_data column exists: {}", e))?;
        
        if column_exists.is_none() {
            sqlx::query("ALTER TABLE apps ADD COLUMN icon_data BLOB")
                .execute(&pool)
                .await
                .map_err(|e| format!("Failed to add icon_data column: {}", e))?;
        }
    }
    
    *pool_guard = Some(pool.clone());
    Ok(pool)
}

pub fn is_cache_ready_sync() -> Result<bool, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    rt.block_on(async {
        match get_db_pool().await {
            Ok(pool) => {
                let table_check = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='apps'")
                    .fetch_optional(&pool)
                    .await;
                
                match table_check {
                    Ok(Some(_)) => {
                        let result = sqlx::query("SELECT COUNT(*) as count FROM apps")
                            .fetch_one(&pool)
                            .await;
                        match result {
                            Ok(row) => {
                                let count: i64 = row.get("count");
                                Ok(count > 0)
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to query apps count: {}", e);
                                Ok(false)
                            }
                        }
                    }
                    Ok(None) => {
                        Ok(false)
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to check if apps table exists: {}", e);
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to connect to database: {}", e);
                Ok(false)
            }
        }
    })
}

pub async fn get_cached_apps_sync() -> Result<Vec<CachedApp>, String> {
    let pool = get_db_pool().await?;
    
    let table_exists = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='apps'")
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("Failed to check if apps table exists: {}", e))?;
    
    if table_exists.is_none() {
        return Ok(vec![]);
    }
    
    let rows = sqlx::query(
        "SELECT app_id, name, description, summary, download_flatpak_ref, icon_url, icon_path, icon_data, cached_at FROM apps"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query apps: {}", e))?;
    
    Ok(rows.into_iter().map(|row| row_to_cached_app(row)).collect())
}

pub async fn get_cached_app_sync(app_id: String) -> Result<Option<CachedApp>, String> {
    let pool = get_db_pool().await?;
    let row = sqlx::query(
        "SELECT app_id, name, description, summary, download_flatpak_ref, icon_url, icon_path, icon_data, cached_at FROM apps WHERE app_id = ?"
    )
    .bind(&app_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Failed to query app: {}", e))?;
    
    Ok(row.map(row_to_cached_app))
}


pub async fn get_app_icon_data_url_sync(app_id: String) -> Result<Option<String>, String> {
    let pool = get_db_pool().await?;
    let row = sqlx::query("SELECT icon_data FROM apps WHERE app_id = ? AND icon_data IS NOT NULL")
        .bind(&app_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("Failed to query icon data: {}", e))?;
    
    if let Some(row) = row {
        let icon_data: Option<Vec<u8>> = row.get("icon_data");
        if let Some(data) = icon_data {
            let base64 = STANDARD.encode(&data);
            let mime = if data.len() > 4 && &data[0..4] == b"\x89PNG" {
                "image/png"
            } else if data.len() > 2 && &data[0..2] == b"\xff\xd8" {
                "image/jpeg"
            } else if data.len() > 4 && &data[1..5] == b"SVG" {
                "image/svg+xml"
            } else {
                "image/png"
            };
            return Ok(Some(format!("data:{};base64,{}", mime, base64)));
        }
    }
    Ok(None)
}

pub async fn get_cached_apps_batch_sync(app_ids: Vec<String>) -> Result<Vec<CachedApp>, String> {
    get_apps_batch_opt(app_ids, true, true, false).await
}

pub async fn get_apps_batch_opt(
    app_ids: Vec<String>,
    include_description: bool,
    include_icon_data: bool,
    include_cached_at: bool,
) -> Result<Vec<CachedApp>, String> {
    if app_ids.is_empty() {
        return Ok(vec![]);
    }
    
    let pool = get_db_pool().await?;
    let mut columns = vec!["app_id", "name", "summary", "download_flatpak_ref", "icon_url", "icon_path"];
    
    if include_description {
        columns.push("description");
    }
    if include_icon_data {
        columns.push("icon_data");
    }
    if include_cached_at {
        columns.push("cached_at");
    }
    
    let placeholders: Vec<String> = (0..app_ids.len()).map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT {} FROM apps WHERE app_id IN ({})",
        columns.join(", "),
        placeholders.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    for app_id in &app_ids {
        query_builder = query_builder.bind(app_id);
    }
    
    let rows = query_builder
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to query apps batch: {}", e))?;
    
    Ok(rows.into_iter().map(|row| parse_app_row(row, include_description, include_icon_data, include_cached_at)).collect())
}

pub async fn get_cached_categories_sync() -> Result<Vec<CachedCategory>, String> {
    let pool = get_db_pool().await?;
    
    let table_exists = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='categories'")
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("Failed to check if categories table exists: {}", e))?;
    
    if table_exists.is_none() {
        return Ok(vec![]);
    }
    
    let rows = sqlx::query(
        "SELECT id, name, cached_at FROM categories WHERE id IN (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ORDER BY name"
    )
    .bind("AudioVideo")
    .bind("Development")
    .bind("Education")
    .bind("Game")
    .bind("Graphics")
    .bind("Network")
    .bind("Office")
    .bind("Science")
    .bind("System")
    .bind("Utility")
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query categories: {}", e))?;
    
    Ok(rows.into_iter().map(|row| CachedCategory {
        id: row.get("id"),
        name: row.get("name"),
        cached_at: row.get("cached_at"),
    }).collect())
}

pub async fn get_cached_category_collection_sync(category_id: String) -> Result<Option<CachedCategoryCollection>, String> {
    let pool = get_db_pool().await?;
    
    let collection_row = sqlx::query(
        "SELECT category_id, total_hits, cached_at FROM category_collections WHERE category_id = ?"
    )
    .bind(&category_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Failed to query category collection: {}", e))?;
    
    if let Some(row) = collection_row {
        let app_rows = sqlx::query(
            "SELECT app_id FROM category_collection_apps WHERE category_id = ? ORDER BY position"
        )
        .bind(&category_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to query category collection apps: {}", e))?;
        
        let app_ids: Vec<String> = app_rows.into_iter().map(|row| row.get("app_id")).collect();
        
        Ok(Some(CachedCategoryCollection {
            category_id: row.get("category_id"),
            app_ids,
            total_hits: row.get::<i64, _>("total_hits") as usize,
            cached_at: row.get("cached_at"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_category_with_apps_sync(
    category_id: String,
    limit: usize,
) -> Result<Option<(CachedCategoryCollection, Vec<CachedApp>)>, String> {
    let pool = get_db_pool().await?;
    
    let collection_row = sqlx::query(
        "SELECT category_id, total_hits, cached_at FROM category_collections WHERE category_id = ?"
    )
    .bind(&category_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| format!("Failed to query category collection: {}", e))?;
    
    if let Some(row) = collection_row {
        let total_hits: i64 = row.get("total_hits");
        let cached_at: i64 = row.get("cached_at");
        
        let app_rows = sqlx::query(
            "SELECT app_id FROM category_collection_apps WHERE category_id = ? ORDER BY position LIMIT ?"
        )
        .bind(&category_id)
        .bind(limit as i64)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to query category collection apps: {}", e))?;
        
        let app_ids: Vec<String> = app_rows.into_iter().map(|row| row.get("app_id")).collect();
        
        let collection = CachedCategoryCollection {
            category_id: row.get("category_id"),
            app_ids: vec![], 
            total_hits: total_hits as usize,
            cached_at,
        };
        
        let apps = if app_ids.is_empty() {
            vec![]
        } else {
            get_apps_batch_opt(app_ids, false, false, false).await?
        };
        
        Ok(Some((collection, apps)))
    } else {
        Ok(None)
    }
}

pub async fn get_category_apps_page_sync(
    category_id: String,
    limit: usize,
    offset: usize,
) -> Result<(Vec<CachedApp>, usize), String> {
    let pool = get_db_pool().await?;
    
    let count_row = sqlx::query(
        "SELECT COUNT(*) as count FROM category_collection_apps WHERE category_id = ?"
    )
    .bind(&category_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to count category apps: {}", e))?;
    
    let total: i64 = count_row.get("count");
    
    let app_rows = sqlx::query(
        "SELECT app_id FROM category_collection_apps WHERE category_id = ? ORDER BY position LIMIT ? OFFSET ?"
    )
    .bind(&category_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query paginated category apps: {}", e))?;
    
    let app_ids: Vec<String> = app_rows.into_iter().map(|row| row.get("app_id")).collect();
    
    let apps = if app_ids.is_empty() {
        vec![]
    } else {
        get_apps_batch_opt(app_ids, false, false, false).await?
    };
    
    Ok((apps, total as usize))
}

pub async fn search_cached_apps_sync(query: String) -> Result<Vec<SearchResult>, String> {
    let pool = get_db_pool().await?;
    let search_pattern = format!("%{}%", query);
    
    let rows = sqlx::query(
        "SELECT app_id, name, summary, icon_url, icon_path FROM apps WHERE name LIKE ? OR summary LIKE ? OR description LIKE ? COLLATE NOCASE LIMIT 100"
    )
    .bind(&search_pattern)
    .bind(&search_pattern)
    .bind(&search_pattern)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to search apps: {}", e))?;
    
    Ok(rows.into_iter().map(|row| SearchResult {
        app_id: row.get("app_id"),
        name: row.get("name"),
        summary: row.get("summary"),
        icon_url: row.get("icon_url"),
        icon_path: row.get("icon_path"),
    }).collect())
}

pub async fn get_cached_collection_apps_sync(collection_type: String) -> Result<Vec<CachedApp>, String> {
    let pool = get_db_pool().await?;
    
    let category_id = match collection_type.as_str() {
        "popular" => "popular",
        "trending" => "trending",
        "recently-updated" => "recently-updated",
        _ => return Err(format!("Unknown collection type: {}", collection_type)),
    };
    
    let app_rows = sqlx::query(
        "SELECT app_id FROM category_collection_apps WHERE category_id = ? ORDER BY position LIMIT 24"
    )
    .bind(category_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query collection apps: {}", e))?;
    
    let app_ids: Vec<String> = app_rows.into_iter().map(|row| row.get("app_id")).collect();
    
    if app_ids.is_empty() {
        Ok(vec![])
    } else {
        get_apps_batch_opt(app_ids, true, true, false).await
    }
}

pub async fn get_homepage_collections_sync() -> Result<(Vec<CachedApp>, Vec<CachedApp>, Vec<CachedApp>), String> {
    let pool = get_db_pool().await?;
    
    let popular_rows = sqlx::query(
        "SELECT app_id FROM category_collection_apps WHERE category_id = 'popular' ORDER BY position LIMIT 8"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query popular apps: {}", e))?;
    
    let trending_rows = sqlx::query(
        "SELECT app_id FROM category_collection_apps WHERE category_id = 'trending' ORDER BY position LIMIT 8"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query trending apps: {}", e))?;
    
    let updated_rows = sqlx::query(
        "SELECT app_id FROM category_collection_apps WHERE category_id = 'recently-updated' ORDER BY position LIMIT 8"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query recently-updated apps: {}", e))?;
    
    let popular_ids: Vec<String> = popular_rows.into_iter().map(|row| row.get("app_id")).collect();
    let trending_ids: Vec<String> = trending_rows.into_iter().map(|row| row.get("app_id")).collect();
    let updated_ids: Vec<String> = updated_rows.into_iter().map(|row| row.get("app_id")).collect();
    
    let all_ids: Vec<String> = popular_ids.iter()
        .chain(trending_ids.iter())
        .chain(updated_ids.iter())
        .cloned()
        .collect();
    
    let all_apps = if all_ids.is_empty() {
        vec![]
    } else {
        get_apps_batch_opt(all_ids, false, true, false).await?
    };
    
    let apps_map: std::collections::HashMap<String, CachedApp> = all_apps
        .into_iter()
        .map(|app| (app.app_id.clone(), app))
        .collect();
    
    let popular_apps: Vec<CachedApp> = popular_ids.into_iter()
        .filter_map(|id| apps_map.get(&id).cloned())
        .collect();
    
    let trending_apps: Vec<CachedApp> = trending_ids.into_iter()
        .filter_map(|id| apps_map.get(&id).cloned())
        .collect();
    
    let updated_apps: Vec<CachedApp> = updated_ids.into_iter()
        .filter_map(|id| apps_map.get(&id).cloned())
        .collect();
    
    Ok((popular_apps, trending_apps, updated_apps))
}

pub async fn get_app_icons_batch_sync(app_ids: Vec<String>) -> Result<Vec<Option<String>>, String> {
    if app_ids.is_empty() {
        return Ok(vec![]);
    }
    
    let pool = get_db_pool().await?;
    let placeholders: Vec<String> = (0..app_ids.len()).map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT app_id, icon_data FROM apps WHERE app_id IN ({}) AND icon_data IS NOT NULL",
        placeholders.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    for app_id in &app_ids {
        query_builder = query_builder.bind(app_id);
    }
    
    let rows = query_builder
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to query icon data: {}", e))?;
    
    let mut icons_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    for row in rows {
        let app_id: String = row.get("app_id");
        let icon_data: Option<Vec<u8>> = row.get("icon_data");
        if let Some(data) = icon_data {
            let base64 = STANDARD.encode(&data);
            let mime = if data.len() > 4 && &data[0..4] == b"\x89PNG" {
                "image/png"
            } else if data.len() > 2 && &data[0..2] == b"\xff\xd8" {
                "image/jpeg"
            } else if data.len() > 4 && &data[1..5] == b"SVG" {
                "image/svg+xml"
            } else {
                "image/png"
            };
            icons_map.insert(app_id, format!("data:{};base64,{}", mime, base64));
        }
    }
    
    let result: Vec<Option<String>> = app_ids.into_iter()
        .map(|id| icons_map.get(&id).cloned())
        .collect();
    
    Ok(result)
}

fn row_to_cached_app(row: SqliteRow) -> CachedApp {
    parse_app_row(row, true, true, true)
}

fn parse_app_row(
    row: SqliteRow,
    include_description: bool,
    include_icon_data: bool,
    include_cached_at: bool,
) -> CachedApp {
    CachedApp {
        app_id: row.get("app_id"),
        name: row.get("name"),
        description: if include_description { row.try_get("description").ok() } else { None },
        summary: row.get("summary"),
        download_flatpak_ref: row.get("download_flatpak_ref"),
        icon_url: row.get("icon_url"),
        icon_path: row.get("icon_path"),
        icon_data: if include_icon_data { row.try_get("icon_data").ok() } else { None },
        cached_at: if include_cached_at { row.get("cached_at") } else { 0 },
    }
}
