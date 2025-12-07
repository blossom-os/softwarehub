-- Create apps table
CREATE TABLE IF NOT EXISTS apps (
    app_id TEXT PRIMARY KEY,
    name TEXT,
    description TEXT,
    summary TEXT,
    download_flatpak_ref TEXT,
    icon_url TEXT,
    icon_path TEXT,
    cached_at INTEGER NOT NULL
);

-- Create categories table
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cached_at INTEGER NOT NULL
);

-- Create category_collections table
CREATE TABLE IF NOT EXISTS category_collections (
    category_id TEXT PRIMARY KEY,
    total_hits INTEGER NOT NULL,
    cached_at INTEGER NOT NULL
);

-- Create category_collection_apps junction table
CREATE TABLE IF NOT EXISTS category_collection_apps (
    category_id TEXT NOT NULL,
    app_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (category_id, app_id),
    FOREIGN KEY (category_id) REFERENCES category_collections(category_id) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_apps_cached_at ON apps(cached_at);
CREATE INDEX IF NOT EXISTS idx_category_collection_apps_category ON category_collection_apps(category_id);
CREATE INDEX IF NOT EXISTS idx_category_collection_apps_position ON category_collection_apps(category_id, position);

