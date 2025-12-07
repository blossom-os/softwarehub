
import Database from "@tauri-apps/plugin-sql";
import type { CachedApp, CachedCategory, CachedCategoryCollection } from "./cache";

let dbInstance: Database | null = null;

export async function getDatabase(): Promise<Database> {
	console.warn("getDatabase is deprecated - use cache.ts Tauri commands instead");
	if (!dbInstance) {
		dbInstance = await Database.load("sqlite:cache.db");
	}
	return dbInstance;
}

export async function closeDatabase(): Promise<void> {
	if (dbInstance) {
		await dbInstance.close();
		dbInstance = null;
	}
}

export async function loadAppsFromDatabase(): Promise<CachedApp[]> {
	console.warn("loadAppsFromDatabase is deprecated - use getCachedApps() from cache.ts instead");
	const db = await getDatabase();
	const rows = await db.select<Array<{
		app_id: string;
		name: string | null;
		description: string | null;
		summary: string | null;
		download_flatpak_ref: string | null;
		icon_url: string | null;
		icon_path: string | null;
		cached_at: number;
	}>>("SELECT app_id, name, description, summary, download_flatpak_ref, icon_url, icon_path, cached_at FROM apps");
	
	return rows.map(row => ({
		app_id: row.app_id,
		name: row.name || undefined,
		description: row.description || undefined,
		summary: row.summary || undefined,
		download_flatpak_ref: row.download_flatpak_ref || undefined,
		icon_url: row.icon_url || undefined,
		icon_path: row.icon_path || undefined,
		cached_at: row.cached_at,
	}));
}

export async function loadCategoriesFromDatabase(): Promise<CachedCategory[]> {
	console.warn("loadCategoriesFromDatabase is deprecated - use getCachedCategories() from cache.ts instead");
	const db = await getDatabase();
	const rows = await db.select<Array<{
		id: string;
		name: string;
		cached_at: number;
	}>>("SELECT id, name, cached_at FROM categories ORDER BY name");
	
	return rows.map(row => ({
		id: row.id,
		name: row.name,
		cached_at: row.cached_at,
	}));
}

export async function saveAppsToDatabase(apps: CachedApp[]): Promise<void> {
	console.warn("saveAppsToDatabase is deprecated - Rust writes to database automatically when fetching from API");
}

export async function saveCategoriesToDatabase(categories: CachedCategory[]): Promise<void> {
	console.warn("saveCategoriesToDatabase is deprecated - Rust writes to database automatically when fetching from API");
}

export async function saveCategoryCollectionToDatabase(collection: CachedCategoryCollection): Promise<void> {
	console.warn("saveCategoryCollectionToDatabase is deprecated - Rust writes to database automatically when fetching from API");
}

export async function loadCategoryCollectionFromDatabase(
	categoryId: string,
	limit?: number,
	offset?: number
): Promise<CachedCategoryCollection | null> {
	console.warn("loadCategoryCollectionFromDatabase is deprecated - use getCachedCategoryCollection() from cache.ts instead");
	const db = await getDatabase();
	
	const rows = await db.select<Array<{
		category_id: string;
		total_hits: number;
		cached_at: number;
	}>>("SELECT category_id, total_hits, cached_at FROM category_collections WHERE category_id = $1", [categoryId]);
	
	if (rows.length === 0) {
		return null;
	}
	
	const collection = rows[0];
	
	let query = "SELECT app_id FROM category_collection_apps WHERE category_id = $1 ORDER BY position";
	const params: any[] = [categoryId];
	
	if (limit !== undefined) {
		query += " LIMIT $2";
		params.push(limit);
		if (offset !== undefined) {
			query += " OFFSET $3";
			params.push(offset);
		}
	}
	
	const appRows = await db.select<Array<{ app_id: string }>>(query, params);
	
	return {
		category_id: collection.category_id,
		app_ids: appRows.map(row => row.app_id),
		total_hits: collection.total_hits,
		cached_at: collection.cached_at,
	};
}
