use crate::cache::queries::get_db_pool;
use crate::cache::types::*;
use crate::cache::icons::download_and_cache_icon;
use chrono::Utc;
use serde_json::Value;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use tauri::{AppHandle, Emitter};

const FLATHUB_API_BASE: &str = "https://flathub.org/api/v2";
const APPS_PER_PAGE: usize = 250;

pub async fn initiate_cache(app: AppHandle, clear_cache: bool) -> Result<(), String> {
    let app_handle = app.clone();
    eprintln!("Starting cache initialization in background (clear_cache: {})...", clear_cache);
    tauri::async_runtime::spawn(async move {
        eprintln!("Background cache fetch task started");
        let pool = get_db_pool().await?;
        
        if clear_cache {
            eprintln!("Clearing existing cache...");
            sqlx::query("DELETE FROM apps").execute(&pool).await
                .map_err(|e| format!("Failed to clear apps: {}", e))?;
            sqlx::query("DELETE FROM categories").execute(&pool).await
                .map_err(|e| format!("Failed to clear categories: {}", e))?;
            sqlx::query("DELETE FROM category_collections").execute(&pool).await
                .map_err(|e| format!("Failed to clear category_collections: {}", e))?;
            sqlx::query("DELETE FROM category_collection_apps").execute(&pool).await
                .map_err(|e| format!("Failed to clear category_collection_apps: {}", e))?;
            eprintln!("Cache cleared");
        }
        
        if clear_cache {
            if let Err(e) = fetch_all_data(app_handle.clone()).await {
                eprintln!("Cache initialization failed: {}", e);
                let _ = app_handle.emit("cache-progress", serde_json::json!({
                    "stage": "error",
                    "progress": 0,
                    "total": 0,
                    "message": "Cache initialization failed",
                    "details": e
                }));
            } else {
                eprintln!("Cache initialization completed successfully");
            }
        } else {
            if let Err(e) = fetch_updated_data(app_handle.clone()).await {
                eprintln!("Cache update failed: {}", e);
                let _ = app_handle.emit("cache-progress", serde_json::json!({
                    "stage": "error",
                    "progress": 0,
                    "total": 0,
                    "message": "Cache update failed",
                    "details": e
                }));
            } else {
                eprintln!("Cache update completed successfully");
            }
        }
        
        Ok::<(), String>(())
    });
    Ok(())
}

async fn fetch_all_data(app: AppHandle) -> Result<(), String> {
    eprintln!("fetch_all_data: Getting database pool...");
    let pool = get_db_pool().await?;
    eprintln!("fetch_all_data: Database pool obtained");
    
    eprintln!("fetch_all_data: Starting to fetch special collections (priority)...");
    let special_collections = vec!["popular", "trending", "recently-updated"];
    for collection_type in special_collections {
        if let Err(e) = fetch_collection_impl(&app, &pool, collection_type).await {
            eprintln!("Failed to fetch {} collection: {}", collection_type, e);
        }
    }
    eprintln!("fetch_all_data: Special collections fetched");
    
    eprintln!("fetch_all_data: Starting to fetch all apps in background...");
    let app_handle = app.clone();
    let pool_clone = pool.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = fetch_all_apps(&app_handle, &pool_clone).await {
            eprintln!("Failed to fetch all apps: {}", e);
        }
    });
    
    eprintln!("fetch_all_data: Extracting categories from apps...");
    fetch_all_categories(&app, &pool).await?;
    eprintln!("fetch_all_data: Categories extracted");

    eprintln!("fetch_all_data: Starting to fetch category collections...");
    fetch_all_category_collections(&app, &pool).await?;
    eprintln!("fetch_all_data: Category collections fetched");
    
    let app_count = sqlx::query("SELECT COUNT(*) as count FROM apps")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to count apps: {}", e))?;
    let count: i64 = app_count.get("count");
    
    let _ = app.emit("cache-progress", serde_json::json!({
        "stage": "complete",
        "progress": count,
        "total": count,
        "percentage": 100,
        "message": format!("Cache complete! Cached {} apps", count)
    }));
    
    Ok(())
}

async fn fetch_updated_data(app: AppHandle) -> Result<(), String> {
    eprintln!("fetch_updated_data: Getting database pool...");
    let pool = get_db_pool().await?;
    eprintln!("fetch_updated_data: Database pool obtained");
    
    eprintln!("fetch_updated_data: Starting to fetch special collections (priority)...");
    let special_collections = vec!["popular", "trending", "recently-updated"];
    for collection_type in special_collections {
        if let Err(e) = fetch_collection_impl(&app, &pool, collection_type).await {
            eprintln!("Failed to fetch {} collection: {}", collection_type, e);
        }
    }
    eprintln!("fetch_updated_data: Special collections fetched");
    
    eprintln!("fetch_updated_data: Starting to fetch recently updated apps...");
    let app_handle = app.clone();
    let pool_clone = pool.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = fetch_recently_updated_apps(&app_handle, &pool_clone).await {
            eprintln!("Failed to fetch recently updated apps: {}", e);
        }
    });
    
    eprintln!("fetch_updated_data: Starting to fetch category collections...");
    fetch_all_category_collections(&app, &pool).await?;
    eprintln!("fetch_updated_data: Category collections fetched");
    
    let app_count = sqlx::query("SELECT COUNT(*) as count FROM apps")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to count apps: {}", e))?;
    let count: i64 = app_count.get("count");
    
    let _ = app.emit("cache-progress", serde_json::json!({
        "stage": "complete",
        "progress": count,
        "total": count,
        "percentage": 100,
        "message": format!("Cache updated! {} apps cached", count)
    }));
    
    Ok(())
}

async fn fetch_recently_updated_apps(app: &AppHandle, pool: &SqlitePool) -> Result<(), String> {
    eprintln!("fetch_recently_updated_apps: Starting...");
    
    let url = format!("{}/collection/recently-updated", FLATHUB_API_BASE);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch recently updated collection: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch recently updated collection: HTTP {}", response.status()));
    }
    
    let json: Value = response.json().await
        .map_err(|e| format!("Failed to parse recently updated JSON: {}", e))?;
    
    let apps_array = json.get("hits")
        .and_then(|v| v.as_array())
        .ok_or("Expected hits array in collection response")?;
    
    let app_ids: Vec<String> = apps_array
        .iter()
        .filter_map(|app| {
            app.get("app_id")
                .or_else(|| app.get("flatpakAppId"))
                .and_then(|v| v.as_str())
        })
        .map(|s| s.to_string())
        .collect();
    
    eprintln!("fetch_recently_updated_apps: Got {} app IDs", app_ids.len());
    
    let existing_apps = get_existing_apps_batch(pool, &app_ids).await?;
    let existing_map: std::collections::HashMap<String, CachedApp> = existing_apps
        .into_iter()
        .map(|app| (app.app_id.clone(), app))
        .collect();
    
    let mut tasks = Vec::new();
    for app_id in &app_ids {
        let app_id = app_id.clone();
        let url = format!("{}/appstream/{}", FLATHUB_API_BASE, app_id);
        tasks.push(async move {
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        resp.json::<Value>().await.map(|json| (app_id, json))
                    } else {
                        Err(reqwest::Error::from(resp.error_for_status().unwrap_err()))
                    }
                }
                Err(e) => Err(e),
            }
        });
    }
    
    let results = futures::future::join_all(tasks).await;
    let mut updated_batch = Vec::new();
    
    for result in results {
        match result {
            Ok((app_id, app_json)) => {
                match parse_app_from_json(&app_json) {
                    Ok(new_app) => {
                        if let Some(existing) = existing_map.get(&app_id) {
                            if has_app_changed(existing, &new_app) {
                                updated_batch.push(new_app);
                            }
                        } else {
                            updated_batch.push(new_app);
                        }
                    }
                    Err(e) => eprintln!("Failed to parse app JSON for {}: {}", app_id, e),
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch app: {}", e);
            }
        }
    }
    
    if !updated_batch.is_empty() {
        insert_apps_batch(pool, &updated_batch).await?;
        eprintln!("Updated {} apps out of {} checked", updated_batch.len(), app_ids.len());
        
        let mut icon_tasks = Vec::new();
        for app in &updated_batch {
            if let Some(ref icon_url) = app.icon_url {
                let app_id = app.app_id.clone();
                let icon_url = icon_url.clone();
                let pool_clone = pool.clone();
                icon_tasks.push(async move {
                    let _ = download_and_cache_icon(&pool_clone, &app_id, &icon_url).await;
                });
            }
        }
        
        if !icon_tasks.is_empty() {
            futures::future::join_all(icon_tasks).await;
        }
    } else {
        eprintln!("No apps needed updating");
    }
    
    Ok(())
}

fn has_app_changed(existing: &CachedApp, new: &CachedApp) -> bool {
    existing.name != new.name ||
    existing.summary != new.summary ||
    existing.description != new.description ||
    existing.download_flatpak_ref != new.download_flatpak_ref ||
    existing.icon_url != new.icon_url
}

async fn get_existing_apps_batch(pool: &SqlitePool, app_ids: &[String]) -> Result<Vec<CachedApp>, String> {
    if app_ids.is_empty() {
        return Ok(vec![]);
    }
    
    let placeholders: Vec<String> = (0..app_ids.len()).map(|_| "?".to_string()).collect();
    let query = format!(
        "SELECT app_id, name, description, summary, download_flatpak_ref, icon_url FROM apps WHERE app_id IN ({})",
        placeholders.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    for app_id in app_ids {
        query_builder = query_builder.bind(app_id);
    }
    
    let rows = query_builder
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to query existing apps: {}", e))?;
    
    Ok(rows.into_iter().map(|row| CachedApp {
        app_id: row.get("app_id"),
        name: row.get("name"),
        description: row.get("description"),
        summary: row.get("summary"),
        download_flatpak_ref: row.get("download_flatpak_ref"),
        icon_url: row.get("icon_url"),
        icon_path: None,
        icon_data: None,
        cached_at: 0,
    }).collect())
}

async fn fetch_all_apps(app: &AppHandle, pool: &SqlitePool) -> Result<(), String> {
    eprintln!("fetch_all_apps: Starting...");
    let _ = app.emit("cache-progress", serde_json::json!({
        "stage": "fetching_apps",
        "progress": 0,
        "total": 0,
        "message": "Starting to fetch apps..."
    }));
    
    let url = format!("{}/appstream", FLATHUB_API_BASE);
    eprintln!("fetch_all_apps: Fetching app IDs from {}", url);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch app IDs: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch app IDs: HTTP {}", response.status()));
    }
    
    let app_ids: Vec<String> = response.json().await
        .map_err(|e| format!("Failed to parse app IDs JSON: {}", e))?;
    
    eprintln!("fetch_all_apps: Got {} app IDs", app_ids.len());
    let total_apps = app_ids.len();
    let mut total_fetched = 0;
    
    for chunk in app_ids.chunks(APPS_PER_PAGE) {
        let mut batch = Vec::new();
        
        let mut tasks = Vec::new();
        for app_id in chunk {
            let app_id = app_id.clone();
            let url = format!("{}/appstream/{}", FLATHUB_API_BASE, app_id);
            tasks.push(async move {
                match reqwest::get(&url).await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            resp.json::<Value>().await
                        } else {
                            Err(reqwest::Error::from(resp.error_for_status().unwrap_err()))
                        }
                    }
                    Err(e) => Err(e),
                }
            });
        }
        
        let results = futures::future::join_all(tasks).await;
        
        for result in results {
            match result {
                Ok(app_json) => {
                    match parse_app_from_json(&app_json) {
                        Ok(cached_app) => batch.push(cached_app),
                        Err(e) => eprintln!("Failed to parse app JSON: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch app: {}", e);
                }
            }
        }
        
    if !batch.is_empty() {
        insert_apps_batch(pool, &batch).await?;
        total_fetched += batch.len();
        eprintln!("Inserted batch of {} apps, total: {}", batch.len(), total_fetched);
    } else {
        eprintln!("Warning: Batch is empty, no apps to insert");
    }
        
        for app in &batch {
            if let Some(ref icon_url) = app.icon_url {
                let app_id = app.app_id.clone();
                let icon_url = icon_url.clone();
                let pool_clone = pool.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = download_and_cache_icon(&pool_clone, &app_id, &icon_url).await;
                });
            }
        }
        
    }
    
    Ok(())
}

async fn fetch_all_categories(app: &AppHandle, pool: &SqlitePool) -> Result<(), String> {
    let common_categories = vec![
        ("AudioVideo", "Audio & Video"),
        ("Development", "Development"),
        ("Education", "Education"),
        ("Game", "Games"),
        ("Graphics", "Graphics"),
        ("Network", "Network"),
        ("Office", "Office"),
        ("Science", "Science"),
        ("System", "System"),
        ("Utility", "Utility"),
    ];
    
    sqlx::query("DELETE FROM categories")
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to clear old categories: {}", e))?;
    
    let cached_at = Utc::now().timestamp();
    let mut categories = Vec::new();
    
    for (cat_id, cat_name) in common_categories {
        categories.push(CachedCategory {
            id: cat_id.to_string(),
            name: cat_name.to_string(),
            cached_at,
        });
    }
    
    insert_categories_batch(pool, &categories).await?;
    
    Ok(())
}

async fn fetch_all_category_collections(app: &AppHandle, pool: &SqlitePool) -> Result<(), String> {
    let category_rows = sqlx::query("SELECT id FROM categories")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to query categories: {}", e))?;
    
    let category_ids: Vec<String> = category_rows
        .into_iter()
        .map(|row| row.get("id"))
        .collect();
    
    let total = category_ids.len();
    let mut processed = 0;
    
    for category_id in category_ids {
        if let Err(e) = fetch_category_collection_impl(app, pool, &category_id).await {
            eprintln!("Failed to fetch collection for category {}: {}", category_id, e);
        }
        processed += 1;
        
        let _ = app.emit("cache-progress", serde_json::json!({
            "stage": "fetching_collections",
            "progress": processed,
            "total": total,
            "message": format!("Fetched {}/{} collections", processed, total)
        }));
    }
    
    Ok(())
}

pub async fn fetch_category_collection(
    app: &AppHandle,
    pool: &SqlitePool,
    category_id: &str,
) -> Result<(), String> {
    fetch_category_collection_impl(app, pool, category_id).await
}

async fn fetch_category_collection_impl(
    app: &AppHandle,
    pool: &SqlitePool,
    category_id: &str,
) -> Result<(), String> {
    let url = format!("{}/collection/category/{}", FLATHUB_API_BASE, category_id);
    
    let response = loop {
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch category collection: {}", e))?;
        
        if resp.status().as_u16() == 500 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            continue;
        }
        
        if !resp.status().is_success() {
            return Err(format!("Failed to fetch category collection: HTTP {}", resp.status()));
        }
        
        break resp;
    };
    
    let json: Value = response.json().await
        .map_err(|e| format!("Failed to parse category collection JSON: {}", e))?;
    
    let apps_array = json.get("hits")
        .or_else(|| json.get("apps"))
        .and_then(|v| v.as_array())
        .ok_or("Expected hits or apps array in collection")?;
    
    let app_ids: Vec<String> = apps_array
        .iter()
        .filter_map(|app| {
            app.get("app_id")
                .or_else(|| app.get("flatpakAppId"))
                .and_then(|v| v.as_str())
        })
        .map(|s| s.to_string())
        .collect();
    
    let total_hits = json.get("totalHits")
        .and_then(|v| v.as_u64())
        .unwrap_or(app_ids.len() as u64) as usize;
    
    let cached_at = Utc::now().timestamp();
    
    let collection = CachedCategoryCollection {
        category_id: category_id.to_string(),
        app_ids,
        total_hits,
        cached_at,
    };
    
    insert_category_collection(pool, &collection).await?;
    
    Ok(())
}

pub async fn fetch_collection(
    app: &AppHandle,
    pool: &SqlitePool,
    collection_type: &str,
) -> Result<Vec<String>, String> {
    fetch_collection_impl(app, pool, collection_type).await
}

async fn fetch_collection_impl(
    app: &AppHandle,
    pool: &SqlitePool,
    collection_type: &str,
) -> Result<Vec<String>, String> {
    let url = format!("{}/collection/{}", FLATHUB_API_BASE, collection_type);
    
    let response = loop {
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch collection: {}", e))?;
        
        if resp.status().as_u16() == 500 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            continue;
        }
        
        if resp.status().as_u16() == 404 {
            eprintln!("Collection endpoint {} not found (404) - skipping", collection_type);
            return Ok(vec![]);
        }
        
        if !resp.status().is_success() {
            return Err(format!("Failed to fetch collection: HTTP {}", resp.status()));
        }
        
        break resp;
    };
    
    let json: Value = response.json().await
        .map_err(|e| format!("Failed to parse collection JSON: {}", e))?;
    
    let apps_array = json.get("hits")
        .and_then(|v| v.as_array())
        .ok_or("Expected hits array in collection response")?;
    
    let app_ids: Vec<String> = apps_array
        .iter()
        .filter_map(|app| {
            app.get("app_id")
                .or_else(|| app.get("flatpakAppId"))
                .and_then(|v| v.as_str())
        })
        .map(|s| s.to_string())
        .collect();
    
    let total_hits = json.get("totalHits")
        .and_then(|v| v.as_u64())
        .unwrap_or(app_ids.len() as u64) as usize;
    
    let cached_at = Utc::now().timestamp();
    
    let collection = CachedCategoryCollection {
        category_id: collection_type.to_string(),
        app_ids: app_ids.clone(),
        total_hits,
        cached_at,
    };
    
    insert_category_collection(pool, &collection).await?;
    
    Ok(app_ids)
}

fn parse_app_from_json(json: &Value) -> Result<CachedApp, String> {
    let app_id = json.get("app_id")
        .or_else(|| json.get("id"))
        .or_else(|| json.get("flatpakAppId"))
        .and_then(|v| v.as_str())
        .ok_or("Missing app_id, id, or flatpakAppId")?
        .to_string();
    
    let name = json.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let summary = json.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    let download_flatpak_ref = Some(app_id.clone());
    
    let icon_url = json.get("iconDesktopUrl")
        .or_else(|| json.get("icon"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Ok(CachedApp {
        app_id,
        name,
        description,
        summary,
        download_flatpak_ref,
        icon_url,
        icon_path: None,
        icon_data: None,
        cached_at: Utc::now().timestamp(),
    })
}

async fn insert_apps_batch(pool: &SqlitePool, apps: &[CachedApp]) -> Result<(), String> {
    if apps.is_empty() {
        return Ok(());
    }
    
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;
    
    for app in apps {
        sqlx::query(
            "INSERT OR REPLACE INTO apps (app_id, name, description, summary, download_flatpak_ref, icon_url, icon_path, icon_data, cached_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&app.app_id)
        .bind(&app.name)
        .bind(&app.description)
        .bind(&app.summary)
        .bind(&app.download_flatpak_ref)
        .bind(&app.icon_url)
        .bind(&app.icon_path)
        .bind(&app.icon_data)
        .bind(app.cached_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert app: {}", e))?;
    }
    
    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;
    
    Ok(())
}

async fn insert_categories_batch(pool: &SqlitePool, categories: &[CachedCategory]) -> Result<(), String> {
    if categories.is_empty() {
        return Ok(());
    }
    
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;
    
    for cat in categories {
        sqlx::query(
            "INSERT OR REPLACE INTO categories (id, name, cached_at) VALUES (?, ?, ?)"
        )
        .bind(&cat.id)
        .bind(&cat.name)
        .bind(cat.cached_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert category: {}", e))?;
    }
    
    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;
    
    Ok(())
}

async fn insert_category_collection(
    pool: &SqlitePool,
    collection: &CachedCategoryCollection,
) -> Result<(), String> {
    let mut tx = pool.begin().await
        .map_err(|e| format!("Failed to begin transaction: {}", e))?;
    
    sqlx::query(
        "INSERT OR REPLACE INTO category_collections (category_id, total_hits, cached_at) VALUES (?, ?, ?)"
    )
    .bind(&collection.category_id)
    .bind(collection.total_hits as i64)
    .bind(collection.cached_at)
    .execute(&mut *tx)
    .await
        .map_err(|e| format!("Failed to insert category collection: {}", e))?;
    
    sqlx::query(
        "DELETE FROM category_collection_apps WHERE category_id = ?"
    )
    .bind(&collection.category_id)
    .execute(&mut *tx)
    .await
        .map_err(|e| format!("Failed to delete old app associations: {}", e))?;
    
    for (position, app_id) in collection.app_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO category_collection_apps (category_id, app_id, position) VALUES (?, ?, ?)"
        )
        .bind(&collection.category_id)
        .bind(app_id)
        .bind(position as i64)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert app association: {}", e))?;
    }
    
    tx.commit().await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;
    
    Ok(())
}
