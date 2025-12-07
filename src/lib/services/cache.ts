import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const iconDataUrlCache = new Map<string, string | null>();

export interface CachedApp {
	app_id: string;
	name?: string;
	description?: string;
	summary?: string;
	download_flatpak_ref?: string;
	icon_url?: string;
	icon_path?: string;
	icon_data?: Uint8Array;
	cached_at: number;
}

export interface CachedCategory {
	id: string;
	name: string;
	cached_at: number;
}

export interface CachedCategoryCollection {
	category_id: string;
	app_ids: string[];
	total_hits: number;
	cached_at: number;
}

export interface SearchResult {
	app_id: string;
	name?: string;
	summary?: string;
	icon_url?: string;
	icon_path?: string;
}

export interface CacheProgress {
	stage: string;
	progress: number;
	total: number;
	message: string;
	details?: string;
	appCount?: number;
	categoryId?: string;
}

async function fetchFromAPI<T>(endpoint: string): Promise<T> {
	const response = await fetch(`https://flathub.org/api/v2${endpoint}`);
	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}
	return response.json();
}

export async function getCachedApps(): Promise<CachedApp[]> {
	if (!isTauri) {
		try {
			return await fetchFromAPI<CachedApp[]>("/appstream");
		} catch (error) {
			console.error("Failed to fetch apps from API:", error);
			return [];
		}
	}

	try {
		return await invoke<CachedApp[]>("get_cached_apps_sync");
	} catch (error) {
		console.error("Failed to get cached apps:", error);
		return [];
	}
}

export async function getCachedAppsBatch(appIds: string[]): Promise<CachedApp[]> {
	if (isTauri) {
		try {
			return await invoke<CachedApp[]>("get_apps_batch_opt", {
				appIds,
				includeDescription: true,
				includeIconData: true,
				includeCachedAt: false,
			});
		} catch (error) {
			console.error("Failed to get cached apps batch:", error);
			return [];
		}
	}

	try {
		const apps = await Promise.all(
			appIds.map((id) => fetchFromAPI<CachedApp>(`/apps/${id}`).catch(() => null))
		);
		return apps.filter((app): app is CachedApp => app !== null);
	} catch (error) {
		console.error("Failed to fetch apps batch from API:", error);
		return [];
	}
}

export const getCachedAppsBatchSync = getCachedAppsBatch;

export async function isCacheReady(): Promise<boolean> {
	if (!isTauri) {
		return true;
	}

	try {
		return await invoke<boolean>("is_cache_ready_sync");
	} catch (error) {
		console.error("Failed to check cache readiness:", error);
		return false;
	}
}

export async function getCachedApp(appId: string): Promise<CachedApp | null> {
	if (!isTauri) {
		try {
			return await fetchFromAPI<CachedApp>(`/apps/${appId}`);
		} catch (error) {
			console.error(`Failed to fetch app ${appId} from API:`, error);
			return null;
		}
	}

	try {
		return await invoke<CachedApp | null>("get_cached_app_sync", { appId });
	} catch (error) {
		console.error(`Failed to get cached app ${appId}:`, error);
		return null;
	}
}

export async function getCachedCategories(): Promise<CachedCategory[]> {
	if (!isTauri) {
		try {
			return await fetchFromAPI<CachedCategory[]>("/categories");
		} catch (error) {
			console.error("Failed to fetch categories from API:", error);
			return [];
		}
	}

	try {
		return await invoke<CachedCategory[]>("get_cached_categories_sync");
	} catch (error) {
		console.error("Failed to get cached categories:", error);
		return [];
	}
}

export const getCachedCategoriesSync = getCachedCategories;

export async function getCachedCategoryCollection(
	categoryId: string
): Promise<CachedCategoryCollection | null> {
	if (!isTauri) {
		try {
			const data = await fetchFromAPI<{ apps: CachedApp[]; totalHits: number }>(
				`/apps/collection/category/${categoryId}`
			);
			return {
				category_id: categoryId,
				app_ids: data.apps.map((app) => app.app_id),
				total_hits: data.totalHits,
				cached_at: Date.now(),
			};
		} catch (error) {
			console.error(`Failed to fetch category collection ${categoryId} from API:`, error);
			return null;
		}
	}

	try {
		return await invoke<CachedCategoryCollection | null>("get_cached_category_collection_sync", {
			categoryId,
		});
	} catch (error) {
		console.error(`Failed to get cached category collection ${categoryId}:`, error);
		return null;
	}
}

export const getCachedCategoryCollectionSync = getCachedCategoryCollection;

export async function getCachedCategoryCollectionWithApps(
	categoryId: string,
	limit: number
): Promise<{ collection: CachedCategoryCollection; apps: CachedApp[] } | null> {
	if (!isTauri) {
		try {
			const data = await fetchFromAPI<{ apps: CachedApp[]; totalHits: number }>(
				`/apps/collection/category/${categoryId}?limit=${limit}`
			);
			const collection: CachedCategoryCollection = {
				category_id: categoryId,
				app_ids: data.apps.map((app) => app.app_id),
				total_hits: data.totalHits,
				cached_at: Date.now(),
			};
			return { collection, apps: data.apps };
		} catch (error) {
			console.error(`Failed to fetch category collection ${categoryId} from API:`, error);
			return null;
		}
	}

	try {
		const result = await invoke<[CachedCategoryCollection, CachedApp[]] | null>(
			"get_cached_category_collection_with_apps_sync",
			{
				categoryId,
				limit,
			}
		);
		if (result) {
			return { collection: result[0], apps: result[1] };
		}
		return null;
	} catch (error) {
		console.error(`Failed to get cached category collection with apps ${categoryId}:`, error);
		return null;
	}
}

export async function searchCachedApps(query: string): Promise<SearchResult[]> {
	if (!isTauri) {
		try {
			return await fetchFromAPI<SearchResult[]>(`/apps/search?q=${encodeURIComponent(query)}`);
		} catch (error) {
			console.error("Failed to search apps from API:", error);
			return [];
		}
	}

	try {
		return await invoke<SearchResult[]>("search_cached_apps_sync", { query });
	} catch (error) {
		console.error("Failed to search cached apps:", error);
		return [];
	}
}

export async function getCachedCategoryAppsPaginated(
	categoryId: string,
	limit: number,
	offset: number
): Promise<{ apps: CachedApp[]; total: number }> {
	if (!isTauri) {
		try {
			const data = await fetchFromAPI<{ apps: CachedApp[]; totalHits: number }>(
				`/apps/collection/category/${categoryId}?limit=${limit}&offset=${offset}`
			);
			return { apps: data.apps, total: data.totalHits };
		} catch (error) {
			console.error(`Failed to fetch category apps paginated for ${categoryId}:`, error);
			return { apps: [], total: 0 };
		}
	}

	try {
		const result = await invoke<[CachedApp[], number]>("get_cached_category_apps_paginated", {
			categoryId,
			limit,
			offset,
		});
		return { apps: result[0], total: result[1] };
	} catch (error) {
		console.error(`Failed to get cached category apps paginated for ${categoryId}:`, error);
		return { apps: [], total: 0 };
	}
}

export async function getCachedCollectionApps(
	collectionType: "popular" | "trending" | "recently-updated"
): Promise<CachedApp[]> {
	if (!isTauri) {
		try {
			return await fetchFromAPI<CachedApp[]>(`/apps/collection/${collectionType}`);
		} catch (error) {
			console.error(`Failed to get ${collectionType} collection apps from API:`, error);
			return [];
		}
	}

	try {
		return await invoke<CachedApp[]>("get_cached_collection_apps_sync", { collectionType });
	} catch (error) {
		console.error(`Failed to get ${collectionType} collection apps:`, error);
		return [];
	}
}

export const getCachedCollectionAppsSync = getCachedCollectionApps;

export async function getHomepageCollections(): Promise<{
	popular: CachedApp[];
	trending: CachedApp[];
	recentlyUpdated: CachedApp[];
}> {
	if (!isTauri) {
		try {
			const [popular, trending, updated] = await Promise.all([
				fetchFromAPI<CachedApp[]>(`/apps/collection/popular`),
				fetchFromAPI<CachedApp[]>(`/apps/collection/trending`),
				fetchFromAPI<CachedApp[]>(`/apps/collection/recently-updated`),
			]);
			return { popular, trending, recentlyUpdated: updated };
		} catch (error) {
			console.error("Failed to fetch homepage collections from API:", error);
			return { popular: [], trending: [], recentlyUpdated: [] };
		}
	}

	try {
		const result = await invoke<[CachedApp[], CachedApp[], CachedApp[]]>("get_homepage_collections_sync");
		return {
			popular: result[0],
			trending: result[1],
			recentlyUpdated: result[2],
		};
	} catch (error) {
		console.error("Failed to get homepage collections:", error);
		return { popular: [], trending: [], recentlyUpdated: [] };
	}
}

export async function getAppIconsBatch(appIds: string[]): Promise<(string | null)[]> {
	if (!isTauri) {
		return appIds.map(() => null);
	}

	const uncachedIds: string[] = [];
	const cachedResults: (string | null)[] = [];
	
	for (const appId of appIds) {
		if (iconDataUrlCache.has(appId)) {
			cachedResults.push(iconDataUrlCache.get(appId)!);
		} else {
			uncachedIds.push(appId);
			cachedResults.push(null);
		}
	}
	
	if (uncachedIds.length === 0) {
		return cachedResults;
	}

	try {
		const fetchedIcons = await invoke<(string | null)[]>("get_app_icons_batch_sync", { appIds: uncachedIds });
		
		let uncachedIndex = 0;
		for (let i = 0; i < appIds.length; i++) {
			if (cachedResults[i] === null) {
				const iconUrl = fetchedIcons[uncachedIndex];
				cachedResults[i] = iconUrl;
				iconDataUrlCache.set(appIds[i], iconUrl);
				uncachedIndex++;
			}
		}
		
		return cachedResults;
	} catch (error) {
		console.error("Failed to get app icons batch:", error);
		return appIds.map(() => null);
	}
}

export function getCachedIconDataUrl(appId: string): string | null | undefined {
	return iconDataUrlCache.get(appId);
}

export function clearIconCache(): void {
	iconDataUrlCache.clear();
}

export async function initializeCache(onProgress?: (progress: CacheProgress) => void, clearCache: boolean = false): Promise<void> {
	if (!isTauri) {
		return;
	}

	try {
		if (onProgress) {
			const unlisten = await listen<CacheProgress>("cache-progress", (event) => {
				onProgress(event.payload);
			});
		}

		invoke("initiate_cache", { clearCache }).catch((error) => {
			console.error("Cache initialization error:", error);
			onProgress?.({
				stage: "error",
				progress: 0,
				total: 0,
				message: "Cache initialization failed",
				details: String(error),
			});
		});
	} catch (error) {
		console.error("Cache initialization error:", error);
		onProgress?.({
			stage: "error",
			progress: 0,
			total: 0,
			message: "Cache initialization failed",
			details: String(error),
		});
	}
}

export async function cacheApps(apps: CachedApp[]): Promise<void> {
	if (!isTauri) {
		return;
	}

	try {
		await invoke("cache_apps", { apps });
	} catch (error) {
		console.error("Failed to cache apps:", error);
	}
}
