<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { X, ArrowsInSimple, ArrowsOutSimple, Minus } from "phosphor-svelte";

    let kdeTheme = $state<any>(null);
    let appWindow: any = null;
    let isMaximized = $state(false);
    let isFullscreen = $state(false);

    onMount(async () => {
        appWindow = getCurrentWindow();
        try {
            kdeTheme = await invoke("get_kde_theme");
        } catch {
            kdeTheme = { colors: { titlebar_bg: "#3daee9" }, button_icons: {} };
        }

        if (appWindow) {
            isMaximized = await appWindow.isMaximized();
            isFullscreen = await appWindow.isFullscreen();

            await appWindow.onResized(async () => {
                if (appWindow) {
                    isMaximized = await appWindow.isMaximized();
                    isFullscreen = await appWindow.isFullscreen();
                }
            });

            await appWindow.onMoved(async () => {
                if (appWindow) {
                    isMaximized = await appWindow.isMaximized();
                    isFullscreen = await appWindow.isFullscreen();
                }
            });
        }
    });
</script>

<div
    class="fixed top-0 left-0 right-0 w-screen h-[30px] z-10000 flex items-center select-none text-white"
    data-tauri-drag-region
    style="background-color: {kdeTheme?.colors?.titlebar_bg || '#3daee9'};"
>
    <div class="flex items-center gap-2 p-2 flex-1" data-tauri-drag-region>
        <div class="w-4 h-4">
            <img
                src="/favicon.png"
                alt=""
                class="w-full h-full object-contain"
                data-tauri-drag-region
            />
        </div>
        <div class="text-xs truncate mt-1" data-tauri-drag-region>
            Software Hub
        </div>
    </div>
    <div class="flex items-center h-full shrink-0 m-2">
        <button
            class="w-5 h-5 flex items-center justify-center rounded-md bg-transparent border-0 m-0.5 transition-all duration-150 cursor-pointer appearance-none outline-none hover:bg-white/20"
            onclick={() => appWindow?.minimize()}
            title="Minimize"
        >
            <Minus weight="bold" class="w-3.5 h-3.5" />
        </button>
        <button
            class="w-5 h-5 flex items-center justify-center rounded-md bg-transparent border-0 m-0.5 transition-all duration-150 cursor-pointer appearance-none outline-none hover:bg-white/20"
            onclick={() => appWindow?.toggleMaximize()}
            title={isMaximized || isFullscreen ? "Restore" : "Maximize"}
        >
            {#if isMaximized || isFullscreen}
                <ArrowsInSimple weight="bold" class="w-3.5 h-3.5" />
            {:else}
                <ArrowsOutSimple weight="bold" class="w-3.5 h-3.5" />
            {/if}
        </button>
        <button
            class="w-5 h-5 flex items-center justify-center rounded-md bg-transparent border-0 m-0.5 transition-all duration-150 cursor-pointer appearance-none outline-none hover:bg-red-300/90"
            onclick={() => appWindow?.close()}
            title="Close"
        >
            <X weight="bold" class="w-3.5 h-3.5" />
        </button>
    </div>
</div>
