-- Add indexes for search performance
CREATE INDEX IF NOT EXISTS idx_apps_name ON apps(name);
CREATE INDEX IF NOT EXISTS idx_apps_summary ON apps(summary);
CREATE INDEX IF NOT EXISTS idx_category_collection_apps_category_position ON category_collection_apps(category_id, position);


