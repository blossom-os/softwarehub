<script lang="ts">
	import { onMount } from "svelte";
	import { fly } from "svelte/transition";
	import { overlayState, closeOverlay } from "$lib/stores/overlay";
	import { flathub, isFlatpakInstalled, type App } from "$lib/services/flathub";
	import Button from "$lib/components/ui/button/button.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { convertIconPath } from "$lib/utils";

	let app = $state<App | null>(null);
	let loading = $state(true);
	let installing = $state(false);
	let isInstalled = $state(false);
	let checking = $state(true);
	let imagesReady = $state(false);
	let iconSrc = $state<string | null>(null);

	$effect(() => {
		const unsubscribe = overlayState.subscribe((state) => {
			if (state.appId) {
				loadApp(state.appId);
			} else {
				app = null;
				loading = true;
			}
		});
		return unsubscribe;
	});

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
		if (app) {
			updateIcon();
		}
	});

	async function loadApp(appId: string) {
		loading = true;
		app = null;
		checking = true;
		imagesReady = false;

		try {
			const loadedApp = await flathub.getApp(appId);
			app = loadedApp;

			if (loadedApp) {
				try {
					isInstalled = await isFlatpakInstalled(loadedApp.download_flatpak_ref || loadedApp.app_id);
				} catch (error) {
					console.error("Failed to check if app is installed:", error);
				} finally {
					checking = false;
				}
			} else {
				checking = false;
			}
		} catch (error) {
			console.error("Failed to load app:", error);
			checking = false;
		} finally {
			loading = false;
			setTimeout(() => {
				imagesReady = true;
			}, 100);
		}
	}

	async function handleInstall() {
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
				alert(`Installation failed: ${error}`);
			} finally {
				installing = false;
			}
		}
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			closeOverlay();
		}
	}

	function handleEscape(event: KeyboardEvent) {
		if (event.key === "Escape") {
			closeOverlay();
		}
	}
</script>

<svelte:window onkeydown={handleEscape} />

{#if $overlayState.appId}
	<div
		class="fixed inset-0 z-50 bg-white dark:bg-gray-900 overflow-y-auto"
		transition:fly={{ y: 20, duration: 200 }}
	>
		<div class="min-h-screen">
			<div class="sticky top-0 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 p-4 flex items-center justify-between z-10 shadow-sm">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
					{app?.name || app?.app_id || "Loading..."}
				</h2>
				<Button variant="ghost" onclick={closeOverlay} class="ml-auto">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</Button>
			</div>

			<div class="container mx-auto p-6 max-w-4xl">
				{#if loading || !app}
					<div class="grid md:grid-cols-3 gap-6">
						<div class="md:col-span-1">
							<div class="border border-gray-200 dark:border-gray-800 rounded-lg bg-white dark:bg-gray-900 p-6">
								<div class="w-full rounded-lg mb-4 bg-gray-100 dark:bg-gray-800 flex items-center justify-center min-h-[200px]">
									<div class="w-full h-[200px] bg-gray-200 dark:bg-gray-700 animate-pulse rounded-lg"></div>
								</div>
								<Button class="w-full" disabled={true}>
									<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
									</svg>
									{checking ? "Checking..." : "Loading..."}
								</Button>
							</div>
						</div>
						<div class="md:col-span-2">
							<div class="border border-gray-200 dark:border-gray-800 rounded-lg bg-white dark:bg-gray-900 shadow-sm">
								<div class="flex flex-col space-y-1.5 p-6">
									<div class="h-8 bg-gray-200 dark:bg-gray-700 animate-pulse rounded w-3/4"></div>
								</div>
								<div class="p-6 pt-0">
									<div class="h-6 bg-gray-200 dark:bg-gray-700 animate-pulse rounded w-full mb-4"></div>
									<div class="space-y-2 mb-6">
										<div class="h-4 bg-gray-200 dark:bg-gray-700 animate-pulse rounded w-full"></div>
										<div class="h-4 bg-gray-200 dark:bg-gray-700 animate-pulse rounded w-5/6"></div>
										<div class="h-4 bg-gray-200 dark:bg-gray-700 animate-pulse rounded w-4/6"></div>
									</div>
								</div>
							</div>
						</div>
					</div>
				{:else}
					<div class="grid md:grid-cols-3 gap-6">
						<div class="md:col-span-1">
							<div class="border border-gray-200 dark:border-gray-800 rounded-lg bg-white dark:bg-gray-900 p-6">
								{#if iconSrc}
									<div class="w-full rounded-lg mb-4 bg-gray-100 dark:bg-gray-800 flex items-center justify-center min-h-[200px]">
										{#if imagesReady}
											<enhanced:img
												src={iconSrc}
												alt={app.name || app.app_id}
												class="w-full rounded-lg"
												loading="lazy"
												decoding="async"
											/>
										{:else}
											<div class="w-full h-[200px] bg-gray-200 dark:bg-gray-700 animate-pulse rounded-lg"></div>
										{/if}
									</div>
								{:else}
									<div class="w-full rounded-lg mb-4 bg-gray-100 dark:bg-gray-800 flex items-center justify-center min-h-[200px]">
										<div class="w-16 h-16 bg-gray-200 dark:bg-gray-700 rounded-lg"></div>
									</div>
								{/if}
								<Button
									class="w-full {isInstalled ? 'bg-red-600 hover:bg-red-700' : ''}"
									onclick={handleInstall}
									disabled={installing || checking}
								>
									<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
									</svg>
									{checking ? "Checking..." : installing ? (isInstalled ? "Uninstalling..." : "Installing...") : (isInstalled ? "Uninstall" : "Install")}
								</Button>
								{#if app.homepage}
									<Button variant="outline" class="w-full mt-2" onclick={() => app && window.open(app.homepage, "_blank")}>
										<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
										</svg>
										Visit Homepage
									</Button>
								{/if}
							</div>
						</div>

						<div class="md:col-span-2">
							<div class="border border-gray-200 dark:border-gray-800 rounded-lg bg-white dark:bg-gray-900 shadow-sm">
								<div class="flex flex-col space-y-1.5 p-6">
									<h3 class="text-3xl font-semibold leading-none tracking-tight text-gray-900 dark:text-gray-100">
										{app.name || app.app_id}
									</h3>
								</div>
								<div class="p-6 pt-0">
									{#if app.summary}
										<p class="text-lg mb-4 text-gray-900 dark:text-gray-100">{app.summary}</p>
									{/if}

									{#if app.description}
										<div class="mb-6">
											<h3 class="font-semibold mb-2 text-gray-900 dark:text-gray-100">Description</h3>
											<p class="text-gray-600 dark:text-gray-400 whitespace-pre-wrap">{app.description}</p>
										</div>
									{/if}

									<div class="grid grid-cols-2 gap-4 mb-6">
										{#if app.developer_name}
											<div>
												<p class="text-sm font-semibold text-gray-900 dark:text-gray-100">Developer</p>
												<p class="text-gray-600 dark:text-gray-400">{app.developer_name}</p>
											</div>
										{/if}
										{#if app.current_release_version}
											<div>
												<p class="text-sm font-semibold text-gray-900 dark:text-gray-100">Version</p>
												<p class="text-gray-600 dark:text-gray-400">{app.current_release_version}</p>
											</div>
										{/if}
										{#if app.project_license}
											<div>
												<p class="text-sm font-semibold text-gray-900 dark:text-gray-100">License</p>
												<p class="text-gray-600 dark:text-gray-400">{app.project_license}</p>
											</div>
										{/if}
									</div>
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

