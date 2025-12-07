use sqlx::sqlite::SqlitePool;
use crate::cache::queries::get_db_pool;

pub async fn download_and_cache_icon(
    pool: &SqlitePool,
    app_id: &str,
    icon_url: &str,
) -> Result<Option<String>, String> {
    let existing = sqlx::query("SELECT icon_data FROM apps WHERE app_id = ? AND icon_data IS NOT NULL")
        .bind(app_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check existing icon: {}", e))?;
    
    if existing.is_some() {
        return Ok(None); 
    }
    
    let response = reqwest::get(icon_url)
        .await
        .map_err(|e| format!("Failed to download icon: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download icon: HTTP {}", response.status()));
    }
    
    let icon_bytes = response.bytes().await
        .map_err(|e| format!("Failed to read icon bytes: {}", e))?;
    
    sqlx::query("UPDATE apps SET icon_data = ? WHERE app_id = ?")
        .bind(icon_bytes.as_ref())
        .bind(app_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update icon in database: {}", e))?;
    
    Ok(Some(format!("Icon cached for {}", app_id)))
}

pub async fn download_and_cache_icon_command(
    app_id: String,
    icon_url: String,
) -> Result<Option<String>, String> {
    let pool = get_db_pool().await?;
    download_and_cache_icon(&pool, &app_id, &icon_url).await
}
