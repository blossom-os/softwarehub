<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { page } from "$app/stores";
	import AppCard from "$lib/components/AppCard.svelte";
	import { type Collection, type App } from "$lib/services/flathub";
	import { goto, afterNavigate } from "$app/navigation";
	import { invoke } from "@tauri-apps/api/core";
	import Button from "$lib/components/ui/button/button.svelte";
	import { getCachedCategoryCollectionWithApps, getCachedCategoryAppsPaginated } from "$lib/services/cache";
	import { convertIconPath } from "$lib/utils";
	import { openAppDetails } from "$lib/stores/overlay";

	let category = $state<Collection | null>(null);
	let apps = $state<App[]>([]);
	let visibleApps = $state<App[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let hasMore = $state(false);
	let currentPage = $state(0);
	const pageSize = 24;
	const initialVisible = 24;
	let containerElement = $state<HTMLDivElement | null>(null);
	
	let sentinelElement = $state<HTMLElement | null>(null);
	let observer: IntersectionObserver | null = null;
	
	async function loadCategory(categoryId: string) {
		loading = true;
		apps = [];
		visibleApps = [];
		hasMore = false;
		currentPage = 0;
		
		category = {
			id: categoryId,
			name: categoryId,
		};
		
		const categoryStr = categoryId.charAt(0).toUpperCase() + categoryId.slice(1);
		const categoryStrLower = categoryId.toLowerCase();
		
		try {
			let result = await getCachedCategoryCollectionWithApps(categoryStr, pageSize);
			
			if (!result || !result.collection || !result.apps || result.apps.length === 0) {
				result = await getCachedCategoryCollectionWithApps(categoryStrLower, pageSize);
			}
			
			if (result && result.collection && result.apps && result.apps.length > 0) {
				const appMap = new Map<string, App>();
				result.apps.forEach((cached) => {
					if (!appMap.has(cached.app_id)) {
						const icon = convertIconPath(cached.icon_path) || cached.icon_url;
						appMap.set(cached.app_id, {
							app_id: cached.app_id,
							name: cached.name || undefined,
							summary: cached.summary || undefined,
							description: cached.description || undefined,
							download_flatpak_ref: cached.download_flatpak_ref || undefined,
							icon: icon || undefined,
						});
					}
				});
				const newApps = Array.from(appMap.values());
				const existingIds = new Set(apps.map(app => app.app_id));
				const uniqueNewApps = newApps.filter(app => !existingIds.has(app.app_id));
				apps = [...apps, ...uniqueNewApps];
				visibleApps = apps.slice(0, initialVisible);
				category = {
					id: categoryId,
					name: categoryId,
					totalHits: result.collection.total_hits,
				};
				hasMore = apps.length < result.collection.total_hits;
				loading = false;
				return;
			}
		} catch (error) {
			console.error("Failed to get cached category collection:", error);
		}
		
		try {
			let result = await getCachedCategoryAppsPaginated(categoryStr, pageSize, 0);
			
			if (!result || !result.apps || result.apps.length === 0) {
				result = await getCachedCategoryAppsPaginated(categoryStrLower, pageSize, 0);
			}
			
			if (result && result.apps && result.apps.length > 0) {
				const appMap = new Map<string, App>();
				result.apps.forEach((cached) => {
					if (!appMap.has(cached.app_id)) {
						const icon = convertIconPath(cached.icon_path) || cached.icon_url;
						appMap.set(cached.app_id, {
							app_id: cached.app_id,
							name: cached.name || undefined,
							summary: cached.summary || undefined,
							description: cached.description || undefined,
							download_flatpak_ref: cached.download_flatpak_ref || undefined,
							icon: icon || undefined,
						});
					}
				});
				const newApps = Array.from(appMap.values());
				const existingIds = new Set(apps.map(app => app.app_id));
				const uniqueNewApps = newApps.filter(app => !existingIds.has(app.app_id));
				apps = [...apps, ...uniqueNewApps];
				visibleApps = apps.slice(0, initialVisible);
				category = {
					id: categoryId,
					name: categoryId,
					totalHits: result.total,
				};
				hasMore = apps.length < result.total;
				loading = false;
				return;
			}
		} catch (error) {
			console.error("Failed to get cached category apps:", error);
		}
		
		invoke("fetch_and_cache_category_collection", { categoryId: categoryStr }).catch((error) => {
			console.error(`Failed to trigger category fetch for ${categoryStr}:`, error);
		});
		
		const { getCurrentWindow } = await import("@tauri-apps/api/window");
		const window = getCurrentWindow();
		let unlistenProgress: (() => void) | null = null;
		
		unlistenProgress = await window.listen("cache-progress", async (event: any) => {
			const progress = event.payload;
			const stage = progress.stage || "";
			
			if (stage === "complete") {
				await loadCategory(categoryId);
				if (unlistenProgress) {
					unlistenProgress();
					unlistenProgress = null;
				}
			}
		});
		
		setTimeout(() => {
			if (unlistenProgress) {
				unlistenProgress();
			}
		}, 30000);
		
		loading = false;
	}
	
	onMount(() => {
		const categoryId = $page.params.id;
		if (!categoryId) {
			loading = false;
			return;
		}
		
		loadCategory(categoryId).catch((error) => {
			console.error("Unhandled error in category load:", error);
			loading = false;
		});
	});
	
	afterNavigate(() => {
		const categoryId = $page.params.id;
		if (categoryId && category?.id !== categoryId) {
			loadCategory(categoryId).catch((error) => {
				console.error("Unhandled error in category load:", error);
				loading = false;
			});
		}
	});

	async function loadMore() {
		if (loadingMore || !hasMore) return;
		
		const categoryId = $page.params.id;
		if (!categoryId) return;
		
		loadingMore = true;
		
		try {
			const currentVisible = visibleApps.length;
			const nextBatch = apps.slice(currentVisible, currentVisible + pageSize);
			
			if (nextBatch.length > 0) {
				const existingIds = new Set(visibleApps.map(app => app.app_id));
				const uniqueBatch = nextBatch.filter(app => !existingIds.has(app.app_id));
				visibleApps = [...visibleApps, ...uniqueBatch];
				hasMore = visibleApps.length < apps.length;
			} else {
				const categoryStr = categoryId.charAt(0).toUpperCase() + categoryId.slice(1);
				currentPage += 1;
				
				const result = await getCachedCategoryAppsPaginated(categoryStr, pageSize, currentPage * pageSize);
				
				if (result.apps.length > 0) {
					const existingAppIds = new Set(apps.map(app => app.app_id));
					const newApps = result.apps
						.filter(cached => !existingAppIds.has(cached.app_id))
						.map((cached) => {
							const icon = convertIconPath(cached.icon_path) || cached.icon_url;
							return {
								app_id: cached.app_id,
								name: cached.name || undefined,
								summary: cached.summary || undefined,
								description: cached.description || undefined,
								download_flatpak_ref: cached.download_flatpak_ref || undefined,
								icon: icon || undefined,
							};
						});
					apps = [...apps, ...newApps];
					const existingVisibleIds = new Set(visibleApps.map(app => app.app_id));
					const uniqueNewApps = newApps.filter(app => !existingVisibleIds.has(app.app_id));
					visibleApps = [...visibleApps, ...uniqueNewApps];
					hasMore = (currentPage * pageSize + result.apps.length) < result.total;
				} else {
					hasMore = false;
				}
			}
		} catch (error) {
			console.error("Failed to load more apps:", error);
			hasMore = false;
		} finally {
			loadingMore = false;
		}
	}

	$effect(() => {
		if (observer) {
			observer.disconnect();
			observer = null;
		}
		
		if (sentinelElement && hasMore && !loadingMore && !loading) {
			observer = new IntersectionObserver(
				(entries) => {
					if (entries[0]?.isIntersecting && hasMore && !loadingMore) {
						loadMore();
					}
				},
				{ rootMargin: "200px" }
			);
			observer.observe(sentinelElement);
		}
		
		return () => {
			if (observer) {
				observer.disconnect();
				observer = null;
			}
		};
	});

	onDestroy(() => {
		if (observer) {
			observer.disconnect();
			observer = null;
		}
	});
</script>

<div class="container mx-auto p-4 pt-8 h-full flex flex-col">
	<Button variant="ghost" onclick={() => goto("/")} class="mb-4">
		<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>
		Back
	</Button>

	{#if category}
		<h1 class="text-3xl font-bold mb-6 text-gray-900 dark:text-gray-100">
			{category.name || category.id || "Category"}
		</h1>
		{#if category.description}
			<p class="text-gray-600 dark:text-gray-400 mb-6">{category.description}</p>
		{/if}
		<div 
			bind:this={containerElement}
			class="flex-1 overflow-y-auto"
		>
			{#if visibleApps.length > 0}
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 pb-8">
					{#each visibleApps as app (app.app_id)}
						<div class="block" role="button" tabindex="0" onclick={() => openAppDetails(app.app_id)}>
							<AppCard {app} />
						</div>
					{/each}
				</div>
			{:else if loading}
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 pb-8">
					{#each Array(12) as _}
						<AppCard skeleton={true} />
					{/each}
				</div>
			{:else}
				<div class="text-center py-12">
					<p class="text-gray-600 dark:text-gray-400">No apps found in this category</p>
				</div>
			{/if}
			{#if hasMore}
				<div bind:this={sentinelElement} class="h-4 flex items-center justify-center py-4">
					{#if loadingMore}
						<div class="animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
					{/if}
				</div>
			{/if}
		</div>
	{:else}
		<div class="text-center py-12">
			<p class="text-gray-600 dark:text-gray-400">Category not found</p>
		</div>
	{/if}
</div>
