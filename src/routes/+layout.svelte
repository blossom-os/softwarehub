<script lang="ts">
    import "../app.css";
    import Titlebar from "$lib/Titlebar.svelte";
    import DownloadCenter from "$lib/components/DownloadCenter.svelte";
    import CacheProgressBar from "$lib/components/CacheProgressBar.svelte";
    import AppDetailsOverlay from "$lib/components/AppDetailsOverlay.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, type Snippet } from "svelte";
    import { type CacheProgress } from "../lib/services/cache";
    
    let { children }: { children: Snippet } = $props();

    let cacheProgress = $state<CacheProgress | null>(null);
    let cacheProgressVisible = $state(false);

    onMount(() => {
        const updateDarkMode = () => {
            if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        };
        updateDarkMode();
        const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
        darkModeQuery.addEventListener('change', updateDarkMode);

        queueMicrotask(() => {
            try {
                const preventDrag = (e: DragEvent) => {
                    e.preventDefault();
                    return false;
                };

                const preventSelect = (e: Event) => {
                    e.preventDefault();
                    return false;
                };

                document.addEventListener('dragstart', preventDrag);
                document.addEventListener('selectstart', preventSelect);
            } catch (error) {
                console.error("Layout setup error:", error);
            }
        });

        queueMicrotask(async () => {
            try {
                await new Promise(resolve => setTimeout(resolve, 300));
                await invoke("set_complete");
            } catch (error) {
                console.error("Error transitioning from splashscreen:", error);
            }
        });

        let unlistenProgress: (() => void) | null = null;
        queueMicrotask(async () => {
            try {
                const { getCurrentWindow } = await import("@tauri-apps/api/window");
                const window = getCurrentWindow();
                
                unlistenProgress = await window.listen<CacheProgress>("cache-progress", async (event) => {
                    const progress = event.payload;
                    
                    if (progress.stage === "complete") {
                        cacheProgress = progress;
                        cacheProgressVisible = true;
                        setTimeout(() => {
                            cacheProgressVisible = false;
                        }, 1000);
                    } else if (progress.stage === "error") {
                        cacheProgress = progress;
                        cacheProgressVisible = true;
                        setTimeout(() => {
                            cacheProgressVisible = false;
                        }, 3000);
                    }
                });
            } catch (error) {
                console.error("Cache initialization error:", error);
                cacheProgressVisible = false;
            }
        });
    });
</script>

<div class="min-h-screen bg-white dark:bg-gray-950 flex flex-col">
    <Titlebar />
    <CacheProgressBar progress={cacheProgress} visible={cacheProgressVisible} />
    <div class="flex-1 w-full overflow-auto" style="margin-top: 30px;">
        {@render children()}
    </div>
    <DownloadCenter />
    <AppDetailsOverlay />
</div>
