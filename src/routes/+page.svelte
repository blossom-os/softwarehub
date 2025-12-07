<script lang="ts">
	import { onMount } from "svelte";
	import AppCard from "$lib/components/AppCard.svelte";
	import { flathub, type App, type Collection } from "$lib/services/flathub";
	import { goto, afterNavigate } from "$app/navigation";
	import { getCachedCategories, getHomepageCollections, getAppIconsBatch, type CachedApp } from "$lib/services/cache";
	import { openAppDetails } from "$lib/stores/overlay";

	let searchQuery = $state("");
	let searchResults = $state<App[]>([]);
	let searching = $state(false);
	let popularApps = $state<App[]>([]);
	let trendingApps = $state<App[]>([]);
	let recentlyUpdated = $state<App[]>([]);
	let categories = $state<Collection[]>([]);
	let hasData = $state(false);

	const DISPLAY_COUNT = 8; // Show only 8 apps per section on frontpage

	const showSearchResults = $derived(searchResults.length > 0);
	const showHomeContent = $derived(!showSearchResults);
	const showSkeletons = $derived(!hasData && !showSearchResults);
	
	function mapAppSync(app: { icon_path?: string; icon_url?: string; app_id: string; name?: string; summary?: string; description?: string; download_flatpak_ref?: string }): App {
		return {
			app_id: app.app_id,
			name: app.name,
			summary: app.summary,
			description: app.description,
			download_flatpak_ref: app.download_flatpak_ref,
			icon: app.icon_url || app.icon_path,
		};
	}

	async function loadCategories() {
		try {
			const cats = await getCachedCategories();
			if (cats && cats.length > 0) {
				categories = cats.map(cat => ({
					id: cat.id,
					name: cat.name,
				}));
			}
		} catch (error) {
			console.error("Failed to load categories from cache:", error);
		}
	}

	onMount(async () => {
		Promise.all([loadCategories(), loadCollections()]);
		
		const { getCurrentWindow } = await import("@tauri-apps/api/window");
		await getCurrentWindow().listen("cache-progress", async (event: any) => {
			const progress = event.payload;
			
			if (progress.stage === "complete") {
				await Promise.all([loadCategories(), loadCollections()]);
			}
		});
	});

	async function loadCollections() {
		try {
			const collections = await getHomepageCollections();
			
			const allAppIdsSet = new Set<string>();
			collections.popular.forEach(app => allAppIdsSet.add(app.app_id));
			collections.trending.forEach(app => allAppIdsSet.add(app.app_id));
			collections.recentlyUpdated.forEach(app => allAppIdsSet.add(app.app_id));
			const allAppIds = Array.from(allAppIdsSet);
			
			const iconDataUrls = await getAppIconsBatch(allAppIds);
			const iconMap = new Map<string, string | null>();
			allAppIds.forEach((id, index) => {
				iconMap.set(id, iconDataUrls[index]);
			});
			
			const appMap = new Map<string, App>();
			const mapApp = (cached: CachedApp): App => {
				if (appMap.has(cached.app_id)) {
					return appMap.get(cached.app_id)!;
				}
				let icon: string | undefined = iconMap.get(cached.app_id) || undefined;
				if (!icon) {
					icon = cached.icon_url || undefined;
				}
				const app: App = {
					app_id: cached.app_id,
					name: cached.name || undefined,
					summary: cached.summary || undefined,
					description: cached.description || undefined,
					download_flatpak_ref: cached.download_flatpak_ref || undefined,
					icon: icon,
				};
				appMap.set(cached.app_id, app);
				return app;
			};
			
			if (collections.popular.length > 0) {
				const uniquePopular = collections.popular
					.filter((app, index, self) => self.findIndex(a => a.app_id === app.app_id) === index)
					.slice(0, DISPLAY_COUNT)
					.map(mapApp);
				popularApps = uniquePopular;
				hasData = true;
			}
			if (collections.trending.length > 0) {
				const uniqueTrending = collections.trending
					.filter((app, index, self) => self.findIndex(a => a.app_id === app.app_id) === index)
					.slice(0, DISPLAY_COUNT)
					.map(mapApp);
				trendingApps = uniqueTrending;
			}
			if (collections.recentlyUpdated.length > 0) {
				const uniqueRecentlyUpdated = collections.recentlyUpdated
					.filter((app, index, self) => self.findIndex(a => a.app_id === app.app_id) === index)
					.slice(0, DISPLAY_COUNT)
					.map(mapApp);
				recentlyUpdated = uniqueRecentlyUpdated;
			}
		} catch (error) {
			console.error("Failed to load collections:", error);
		}
	}

	afterNavigate(async () => {
		await Promise.all([loadCategories(), loadCollections()]);
	});

	function handleCategoryClick(categoryId: string) {
		goto(`/category/${categoryId}`);
	}

	async function handleSearch() {
		if (!searchQuery.trim()) {
			searchResults = [];
			return;
		}
		searching = true;
		try {
			const result = await flathub.searchApps(searchQuery);
			searchResults = result.hits || [];
		} catch (error) {
			console.error("Search failed:", error);
			searchResults = [];
		} finally {
			searching = false;
		}
	}

	function clearSearch() {
		searchQuery = "";
		searchResults = [];
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === "Enter") {
			handleSearch();
		}
	}
</script>

<div class="container mx-auto p-4 pt-4">
	<h1 class="text-3xl font-bold mb-6 text-gray-900 dark:text-gray-100">Software Hub</h1>
	
	{#if categories.length > 0}
		<div class="mb-8">
			<h2 class="text-2xl font-bold mb-4 text-gray-900 dark:text-gray-100">Categories</h2>
			<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 lg:grid-cols-10 gap-3">
				{#each categories as category (category.id)}
					<button
						onclick={() => handleCategoryClick(category.id || "")}
						class="p-4 border border-gray-200 dark:border-gray-800 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors bg-white dark:bg-gray-900 text-left"
					>
						<h3 class="font-semibold text-sm text-gray-900 dark:text-gray-100 truncate">{category.name || category.id}</h3>
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<div class="mb-8">
		<div class="flex gap-2 max-w-2xl">
			<input
				type="text"
				class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-700 rounded-md bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100"
				placeholder="Search for apps..."
				bind:value={searchQuery}
				onkeydown={handleKeydown}
			/>
			<button
				class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
				onclick={handleSearch}
				disabled={searching}
			>
				{searching ? "Searching..." : "Search"}
			</button>
		</div>
	</div>

	{#if showSearchResults}
		<div class="mb-8">
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Search Results</h2>
				<button
					class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
					onclick={clearSearch}
				>
					Clear Search
				</button>
			</div>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{#each searchResults as app (app.app_id)}
					<div class="block" onclick={() => openAppDetails(app.app_id)}>
						<AppCard {app} />
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if popularApps.length > 0 && showHomeContent}
		<div class="mb-8">
			<div class="flex items-center gap-2 mb-4">
				<h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Popular Apps</h2>
			</div>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{#each popularApps as app (app.app_id)}
					<div class="block" role="button" tabindex="0" onclick={() => openAppDetails(app.app_id)}>
						<AppCard {app} />
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if trendingApps.length > 0 && showHomeContent}
		<div class="mb-8">
			<div class="flex items-center gap-2 mb-4">
				<h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Trending</h2>
			</div>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{#each trendingApps as app (app.app_id)}
					<div class="block" role="button" tabindex="0" onclick={() => openAppDetails(app.app_id)}>
						<AppCard {app} />
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if recentlyUpdated.length > 0 && showHomeContent}
		<div class="mb-8">
			<div class="flex items-center gap-2 mb-4">
				<h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Recently Updated</h2>
			</div>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{#each recentlyUpdated as app (app.app_id)}
					<div class="block" role="button" tabindex="0" onclick={() => openAppDetails(app.app_id)}>
						<AppCard {app} />
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if showSkeletons}
		<div class="mb-8">
			<div class="flex items-center gap-2 mb-4">
				<div class="h-7 bg-gray-200 dark:bg-gray-700 rounded animate-pulse w-32"></div>
			</div>
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
				{#each Array(8) as _}
					<AppCard skeleton={true} />
				{/each}
			</div>
		</div>
	{/if}

	{#if !showSkeletons && !hasData && !showSearchResults}
		<div class="text-center py-12">
			<h2 class="text-2xl font-bold mb-2 text-gray-900 dark:text-gray-100">Welcome to Software Hub</h2>
		</div>
	{/if}
</div>
