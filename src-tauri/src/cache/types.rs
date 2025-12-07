use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedApp {
    pub app_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub download_flatpak_ref: Option<String>,
    pub icon_url: Option<String>,
    pub icon_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_data: Option<Vec<u8>>,
    pub cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCategory {
    pub id: String,
    pub name: String,
    pub cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCategoryCollection {
    pub category_id: String,
    pub app_ids: Vec<String>,
    pub total_hits: usize,
    pub cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub app_id: String,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub icon_url: Option<String>,
    pub icon_path: Option<String>,
}
