import { invoke } from "@tauri-apps/api/core";
import {
	getCachedCategories,
	getCachedCategoryCollection,
	getCachedAppsBatch,
	getCachedApp,
	searchCachedApps,
	getCachedCollectionApps,
	getAppIconsBatch,
	getCachedIconDataUrl,
	type CachedApp,
} from "./cache";
import { convertIconPath } from "$lib/utils";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;


const fetchingCollections = new Set<string>();

export type CategoryId = 
	| "AudioVideo"
	| "Development"
	| "Education"
	| "Game"
	| "Graphics"
	| "Network"
	| "Office"
	| "Science"
	| "System"
	| "Utility";

export interface App {
	app_id: string;
	id?: string;
	name?: string;
	summary?: string;
	description?: string;
	homepage?: string;
	download_flatpak_ref?: string;
	current_release_version?: string;
	current_release_date?: string;
	main_categories?: string;
	sub_categories?: string[];
	keywords?: string[];
	categories?: Array<{ name: string }>;
	download_size?: number;
	icon?: string;
	screenshots?: Array<{
		img_mobile_url?: string;
		img_desktop_url?: string;
		thumb_url?: string;
	}>;
	developer_name?: string;
	project_license?: string;
	runtime?: string;
	arches?: string[];
	type?: string;
	added_at?: number | string;
	updated_at?: number | string;
	favorites_count?: number;
	installs_last_month?: number;
	isMobileFriendly?: boolean;
	is_free_license?: boolean;
	trending?: number;
	verification_verified?: boolean;
	verification_method?: string;
	verification_login_name?: string;
	verification_login_provider?: string;
	verification_login_is_organization?: boolean;
	verification_website?: string;
	verification_timestamp?: number | string;
}

export interface Collection {
	id?: CategoryId | string;
	name?: string;
	description?: string;
	hits?: App[];
	apps?: App[];
	subcollections?: Collection[];
	hitsPerPage?: number;
	page?: number;
	processingTimeMs?: number;
	query?: string;
	totalHits?: number;
	totalPages?: number;
	facetDistribution?: any;
	facetStats?: any;
}

export interface SearchResponse {
	hits?: App[];
	query?: string;
	processingTimeMs?: number;
	totalHits?: number;
	hitsPerPage?: number;
	page?: number;
	totalPages?: number;
	facetDistribution?: any;
	facetStats?: any;
}

function normalizeTimestamp(value: number | string | null | undefined): number | undefined {
	if (value === null || value === undefined) return undefined;
	if (typeof value === "number") return value;
	if (typeof value === "string") {
		const parsed = parseInt(value, 10);
		return isNaN(parsed) ? undefined : parsed;
	}
	return undefined;
}

function normalizeApp(app: any): App {
	return {
		...app,
		added_at: normalizeTimestamp(app.added_at),
		updated_at: normalizeTimestamp(app.updated_at),
		verification_timestamp: normalizeTimestamp(app.verification_timestamp),
	};
}

function normalizeCollection(collection: any): Collection {
	const normalized: Collection = {
		...collection,
		hits: collection.hits?.map(normalizeApp),
		apps: collection.apps?.map(normalizeApp),
		subcollections: collection.subcollections?.map(normalizeCollection),
		hitsPerPage: collection.hitsPerPage ?? collection.hits_per_page,
		processingTimeMs: collection.processingTimeMs ?? collection.processing_time_ms,
		totalHits: collection.totalHits ?? collection.total_hits,
		totalPages: collection.totalPages ?? collection.total_pages,
	};
	return normalized;
}

async function enrichAppsFromCache(appIds: string[], skipIconData = false): Promise<App[]> {
	const cachedApps = await getCachedAppsBatch(appIds);
	const appsMap = new Map(cachedApps.map(app => [app.app_id, app]));
	
	if (skipIconData) {
		return Promise.all(appIds.map(async (appId) => {
			const cached = appsMap.get(appId);
			if (cached) {
				let icon: string | undefined = undefined;
				
				if (isTauri && cached.icon_data) {
					try {
						const iconDataUrl = await invoke<string | null>("get_app_icon_data_url_sync", { appId });
						if (iconDataUrl) {
							icon = iconDataUrl;
						}
					} catch (error) {
						console.error(`Failed to get icon data URL for ${appId}:`, error);
					}
				}
				
				if (!icon) {
					icon = cached.icon_url || undefined;
				}
				
				return {
					app_id: appId,
					name: cached.name || undefined,
					summary: cached.summary || undefined,
					description: cached.description || undefined,
					download_flatpak_ref: cached.download_flatpak_ref || undefined,
					icon: icon,
				};
			}
			return { app_id: appId };
		}));
	}
	
		const enriched = await Promise.all(appIds.map(async (appId) => {
			const cached = appsMap.get(appId);
			if (cached) {
				let icon: string | undefined = undefined;
				
				if (isTauri && cached.icon_data) {
					const cachedIconUrl = getCachedIconDataUrl(appId);
					if (cachedIconUrl !== undefined) {
						icon = cachedIconUrl || undefined;
					} else {
						const iconDataUrls = await getAppIconsBatch([appId]);
						icon = iconDataUrls[0] || undefined;
					}
				}
				
				if (!icon) {
					icon = convertIconPath(cached.icon_path) || cached.icon_url || undefined;
				}
			
			return {
				app_id: appId,
				name: cached.name || undefined,
				summary: cached.summary || undefined,
				description: cached.description || undefined,
				download_flatpak_ref: cached.download_flatpak_ref || undefined,
				icon: icon,
			};
		}
		return { app_id: appId };
	}));
	
	return enriched;
}


export const flathub = {
	async getAppPicks(): Promise<App[]> {
		return [];
	},

	async getAppPick(id: string): Promise<App> {
		return this.getApp(id);
	},

	async getCollections(): Promise<Collection[]> {
		return [];
	},

	async getCollection(id: string): Promise<Collection> {
		return {
			id,
			hits: [],
			apps: [],
			totalHits: 0,
		};
	},

	async getCollectionCategories(): Promise<Collection[]> {
		const cached = await getCachedCategories();
		
		if (cached && cached.length > 0) {
			return cached.map(cat => ({
				id: cat.id,
				name: cat.name,
			} as Collection));
		}
		
		return [];
	},

	async getCollectionCategory(category: CategoryId | string, limit = 24, offset = 0): Promise<Collection> {
		if (category.length === 0) {
			throw new Error("Empty category ID");
		}
		
		const categoryStr = category.charAt(0).toUpperCase() + category.slice(1) as CategoryId;
		
		const cachedCollection = await getCachedCategoryCollection(categoryStr);
		
		if (cachedCollection && cachedCollection.app_ids && cachedCollection.app_ids.length > 0) {
			const allApps = await enrichAppsFromCache(cachedCollection.app_ids);
			const paginatedApps = allApps.slice(offset, offset + limit);
			
			return {
				id: categoryStr,
				hits: paginatedApps,
				apps: paginatedApps,
				totalHits: cachedCollection.total_hits,
			};
		}
		
		invoke("fetch_and_cache_category_collection", { categoryId: categoryStr }).catch((error) => {
			console.error(`Failed to fetch category ${categoryStr}:`, error);
		});
		
		return {
			id: categoryStr,
			hits: [],
			apps: [],
			totalHits: 0,
		};
	},

	async getCollectionCategorySubcategories(category: CategoryId | string): Promise<Collection[]> {
		return [];
	},

	async getCollectionRecentlyUpdated(): Promise<Collection> {
		const cachedApps = await getCachedCollectionApps("recently-updated");
		if (cachedApps.length > 0) {
			const appIds = cachedApps.map(app => app.app_id);
			const iconDataUrls = await getAppIconsBatch(appIds);
			
			const apps = cachedApps.map((cached: CachedApp, index: number) => {
				let icon: string | undefined = iconDataUrls[index] || undefined;
				
				if (!icon) {
					icon = cached.icon_url || undefined;
				}
				
				return {
					app_id: cached.app_id,
					name: cached.name || undefined,
					summary: cached.summary || undefined,
					description: cached.description || undefined,
					download_flatpak_ref: cached.download_flatpak_ref || undefined,
					icon: icon,
				};
			});
			
			return {
				hits: apps,
				apps: apps,
				totalHits: apps.length,
			};
		}
		
		if (!fetchingCollections.has("recently-updated")) {
			fetchingCollections.add("recently-updated");
			invoke("fetch_and_cache_collection", { collectionType: "recently-updated" })
				.catch((error) => {
					console.error("Failed to fetch recently updated collection:", error);
					fetchingCollections.delete("recently-updated"); 
				})
				.finally(() => {
					setTimeout(() => fetchingCollections.delete("recently-updated"), 5000);
				});
		}
		
		return {
			hits: [],
			apps: [],
			totalHits: 0,
		};
	},

	async getCollectionRecentlyAdded(): Promise<Collection> {
		return {
			hits: [],
			apps: [],
			totalHits: 0,
		};
	},

	async getCollectionPopular(): Promise<Collection> {
		const cachedApps = await getCachedCollectionApps("popular");
		if (cachedApps.length > 0) {
			const iconPromises = cachedApps.map(async (cached: CachedApp) => {
				if (isTauri && cached.icon_data) {
					try {
						return await invoke<string | null>("get_app_icon_data_url_sync", { appId: cached.app_id });
					} catch (error) {
						console.error(`Failed to get icon data URL for ${cached.app_id}:`, error);
						return null;
					}
				}
				return null;
			});
			
			const iconDataUrls = await Promise.all(iconPromises);
			
			const apps = cachedApps.map((cached: CachedApp, index: number) => {
				let icon: string | undefined = iconDataUrls[index] || undefined;
				
				if (!icon) {
					icon = cached.icon_url || undefined;
				}
				
				return {
					app_id: cached.app_id,
					name: cached.name || undefined,
					summary: cached.summary || undefined,
					description: cached.description || undefined,
					download_flatpak_ref: cached.download_flatpak_ref || undefined,
					icon: icon,
				};
			});
			
			return {
				hits: apps,
				apps: apps,
				totalHits: apps.length,
			};
		}
		
		if (!fetchingCollections.has("popular")) {
			fetchingCollections.add("popular");
			invoke("fetch_and_cache_collection", { collectionType: "popular" })
				.catch((error) => {
					console.error("Failed to fetch popular collection:", error);
					fetchingCollections.delete("popular"); 
				})
				.finally(() => {
					setTimeout(() => fetchingCollections.delete("popular"), 5000);
				});
		}
		
		return {
			hits: [],
			apps: [],
			totalHits: 0,
		};
	},

	async getCollectionTrending(): Promise<Collection> {
		const cachedApps = await getCachedCollectionApps("trending");
		if (cachedApps.length > 0) {
			const appIds = cachedApps.map(app => app.app_id);
			const iconDataUrls = await getAppIconsBatch(appIds);
			
			const apps = cachedApps.map((cached: CachedApp, index: number) => {
				let icon: string | undefined = iconDataUrls[index] || undefined;
				
				if (!icon) {
					icon = cached.icon_url || undefined;
				}
				
				return {
					app_id: cached.app_id,
					name: cached.name || undefined,
					summary: cached.summary || undefined,
					description: cached.description || undefined,
					download_flatpak_ref: cached.download_flatpak_ref || undefined,
					icon: icon,
				};
			});
			
			return {
				hits: apps,
				apps: apps,
				totalHits: apps.length,
			};
		}
		
		if (!fetchingCollections.has("trending")) {
			fetchingCollections.add("trending");
			invoke("fetch_and_cache_collection", { collectionType: "trending" })
				.catch((error) => {
					console.error("Failed to fetch trending collection:", error);
					fetchingCollections.delete("trending"); 
				})
				.finally(() => {
					setTimeout(() => fetchingCollections.delete("trending"), 5000);
				});
		}
		
		return {
			hits: [],
			apps: [],
			totalHits: 0,
		};
	},

	async searchApps(query: string, limit = 50, offset = 0): Promise<SearchResponse> {
		const results = await searchCachedApps(query);
		const paginatedResults = results.slice(offset, offset + limit);
		
		return {
			hits: paginatedResults.map((result) => ({
				app_id: result.app_id,
				name: result.name,
				summary: result.summary,
				icon: convertIconPath(result.icon_path || result.icon_url),
			})),
			totalHits: results.length,
			query,
		};
	},

	async getApp(id: string): Promise<App> {
		const cached = await getCachedApp(id);
		
		if (cached) {
			let icon: string | undefined = undefined;
			
			if (isTauri && cached.icon_data) {
				const cachedIconUrl = getCachedIconDataUrl(id);
				if (cachedIconUrl !== undefined) {
					icon = cachedIconUrl || undefined;
				} else {
					const iconDataUrls = await getAppIconsBatch([id]);
					icon = iconDataUrls[0] || undefined;
				}
			}
			
			if (!icon) {
				icon = convertIconPath(cached.icon_path) || cached.icon_url || undefined;
			}
			
			return {
				app_id: cached.app_id,
				name: cached.name,
				summary: cached.summary,
				description: cached.description,
				download_flatpak_ref: cached.download_flatpak_ref,
				icon: icon,
			};
		}
		
		return {
			app_id: id,
		};
	},
};

export async function isFlatpakInstalled(refId: string): Promise<boolean> {
	return await invoke<boolean>("is_flatpak_installed", { refId });
}
