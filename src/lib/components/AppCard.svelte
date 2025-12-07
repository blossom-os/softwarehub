<script lang="ts">
	import type { App } from "$lib/services/flathub";
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import { isFlatpakInstalled } from "$lib/services/flathub";
	import { convertIconPath } from "$lib/utils";

	type Props = {
		app?: App;
		skeleton?: boolean;
	};
	
	let { app, skeleton = false }: Props = $props();
	let installing = $state(false);
	let isInstalled = $state(false);
	let checking = $state(true);
	let iconSrc = $state<string | null>(null);
	
	function updateIcon() {
		if (!app?.icon) {
			iconSrc = null;
			return;
		}
		if (app.icon.startsWith("data:")) {
			iconSrc = app.icon;
		} else {
			iconSrc = convertIconPath(app.icon) || app.icon;
		}
	}
	
	$effect(() => {
		if (!skeleton && app) {
			updateIcon();
		}
	});

	onMount(async () => {
		if (skeleton || !app) return;
		try {
			isInstalled = await isFlatpakInstalled(app.download_flatpak_ref || app.app_id);
		} catch (error) {
			console.error("Failed to check if app is installed:", error);
		} finally {
			checking = false;
		}
	});

	async function handleInstall(event: MouseEvent) {
		event.stopPropagation();
		event.preventDefault();
		
		if (!app) return;
		
		if (isInstalled) {
			const confirmed = confirm(`Are you sure you want to uninstall ${app.name || app.app_id}?`);
			if (!confirmed) return;
			
			installing = true;
			try {
				await invoke("uninstall_flatpak", {
					refName: app.download_flatpak_ref || app.app_id,
				});
				isInstalled = false;
			} catch (error) {
				console.error("Uninstall failed:", error);
				alert(`Uninstallation failed: ${error}`);
			} finally {
				installing = false;
			}
		} else {
			installing = true;
			try {
				await invoke("install_flatpak", {
					refName: app.download_flatpak_ref || app.app_id,
				});
				isInstalled = true;
			} catch (error) {
				console.error("Install failed:", error);
				alert(`Installation failed: ${error}`);
			} finally {
				installing = false;
			}
		}
	}
</script>

{#if skeleton}
	<div class="border border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden bg-white dark:bg-gray-900">
		<div class="w-full h-48 bg-gray-200 dark:bg-gray-700 animate-pulse"></div>
		<div class="p-4">
			<div class="h-5 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mb-2 w-3/4"></div>
			<div class="h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mb-1 w-full"></div>
			<div class="h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mb-3 w-5/6"></div>
			<div class="flex items-center justify-between">
				<div class="h-3 bg-gray-200 dark:bg-gray-700 rounded animate-pulse w-1/3"></div>
				<div class="h-8 bg-gray-200 dark:bg-gray-700 rounded animate-pulse w-20"></div>
			</div>
		</div>
	</div>
{:else if app}
	<div class="border border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden hover:shadow-lg transition-shadow bg-white dark:bg-gray-900 cursor-pointer">
		{#if iconSrc}
			<div class="w-full h-48 bg-gray-100 dark:bg-gray-800 flex items-center justify-center overflow-hidden">
				<enhanced:img 
					src={iconSrc} 
					alt={app.name || app.app_id} 
					class="w-full h-full object-contain"
					loading="lazy"
					decoding="async"
					fetchpriority="low"
				/>
			</div>
		{:else}
			<div class="w-full h-48 bg-gray-100 dark:bg-gray-800 flex items-center justify-center">
				<div class="w-16 h-16 bg-gray-200 dark:bg-gray-700 rounded-lg animate-pulse"></div>
			</div>
		{/if}
		<div class="p-4">
			<h3 class="font-semibold text-lg mb-1 truncate text-gray-900 dark:text-gray-100">{app.name || app.app_id}</h3>
			{#if app.summary}
				<p class="text-sm text-gray-600 dark:text-gray-400 mb-3 line-clamp-2">{app.summary}</p>
			{/if}
			<div class="flex items-center justify-between">
				{#if app.developer_name}
					<span class="text-xs text-gray-500 dark:text-gray-400">{app.developer_name}</span>
				{/if}
				{#if !checking}
					<button
						class="px-3 py-1.5 text-sm rounded-md disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1 transition-colors {isInstalled 
							? 'bg-red-600 hover:bg-red-700 text-white' 
							: 'bg-blue-600 hover:bg-blue-700 text-white'}"
						onclick={handleInstall}
						disabled={installing}
					>
						{installing 
							? (isInstalled ? "Uninstalling..." : "Installing...") 
							: (isInstalled ? "Uninstall" : "Install")}
					</button>
				{:else}
					<div class="px-3 py-1.5 text-sm text-gray-400">Checking...</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

