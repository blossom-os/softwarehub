<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";
    import Titlebar from "$lib/Titlebar.svelte";

    let refName = $state("");
    let progress = $state({
        percentage: 0,
        status: "",
        ref: "",
        speed_mbps: 0,
    });
    let operationStatus = $state("");

    let unlistenProgress: (() => void) | null = null;
    let unlistenOperationStarted: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;
    let unlistenUninstallStarted: (() => void) | null = null;
    let unlistenUninstallComplete: (() => void) | null = null;

    onMount(async () => {
        unlistenProgress = await listen("flatpak-progress", (event) => {
            const data = event.payload as {
                percentage: number;
                status: string;
                ref: string;
                speed_mbps: number;
            };
            progress = data;
            console.log(
                `Progress: ${data.percentage}% - ${data.status} (${data.ref}) - ${data.speed_mbps.toFixed(2)} MB/s`,
            );
        });

        unlistenOperationStarted = await listen(
            "flatpak-operation-started",
            (event) => {
                const data = event.payload as {
                    operation_type: string;
                    ref: string;
                };
                operationStatus = `${data.operation_type} on ${data.ref}`;
                console.log(
                    `Operation started: ${data.operation_type} on ${data.ref}`,
                );
            },
        );

        unlistenComplete = await listen("flatpak-install-complete", (event) => {
            const data = event.payload as { ref: string };
            operationStatus = `Install complete: ${data.ref}`;
            console.log(`Install complete: ${data.ref}`);
        });

        unlistenUninstallStarted = await listen(
            "flatpak-uninstall-started",
            (event) => {
                const data = event.payload as { ref: string };
                progress = {
                    percentage: 0,
                    status: "Starting uninstallation...",
                    ref: data.ref,
                    speed_mbps: 0,
                };
                operationStatus = `Uninstalling: ${data.ref}`;
                console.log(`Uninstall started: ${data.ref}`);
            },
        );

        unlistenUninstallComplete = await listen(
            "flatpak-uninstall-complete",
            (event) => {
                const data = event.payload as { ref: string };
                operationStatus = `Uninstall complete: ${data.ref}`;
                console.log(`Uninstall complete: ${data.ref}`);
            },
        );
    });

    onDestroy(() => {
        unlistenProgress?.();
        unlistenOperationStarted?.();
        unlistenComplete?.();
        unlistenUninstallStarted?.();
        unlistenUninstallComplete?.();
    });

    async function install(event: Event) {
        event.preventDefault();
        if (!refName.trim()) {
            alert("Please enter a Flatpak ref name");
            return;
        }

        try {
            progress = {
                percentage: 0,
                status: "Starting...",
                ref: refName,
                speed_mbps: 0,
            };
            operationStatus = "Starting installation...";
            await invoke("install_flatpak", { refName });
        } catch (error) {
            console.error("Error installing:", error);
            operationStatus = `Error: ${error}`;
        }
    }

    async function uninstall(event: Event) {
        event.preventDefault();
        if (!refName.trim()) {
            alert("Please enter a Flatpak ref name");
            return;
        }

        try {
            progress = {
                percentage: 0,
                status: "Starting...",
                ref: refName,
                speed_mbps: 0,
            };
            operationStatus = "Starting uninstallation...";
            await invoke("uninstall_flatpak", { refName });
        } catch (error) {
            console.error("Error uninstalling:", error);
            operationStatus = `Error: ${error}`;
        }
    }
</script>

<Titlebar />

<main class="m-0 pt-[calc(30px+10vh)] flex flex-col justify-center text-center">
    <h1 class="text-center">Software Hub</h1>

    <form class="flex justify-center" onsubmit={install}>
        <input
            id="greet-input"
            class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit text-[#0f0f0f] bg-white transition-[border-color] duration-[0.25s] shadow-[0_2px_2px_rgba(0,0,0,0.2)] outline-none mr-[5px] dark:text-white dark:bg-[#0f0f0f98]"
            placeholder="Enter Flatpak ref (e.g., org.example.App)..."
            bind:value={refName}
        />
        <button
            type="submit"
            class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit text-[#0f0f0f] bg-white transition-[border-color] duration-[0.25s] shadow-[0_2px_2px_rgba(0,0,0,0.2)] outline-none cursor-pointer hover:border-[#396cd8] active:border-[#396cd8] active:bg-[#e8e8e8] dark:text-white dark:bg-[#0f0f0f98] dark:active:bg-[#0f0f0f69] ml-[5px]"
        >
            Install
        </button>
        <button
            type="button"
            onclick={uninstall}
            class="rounded-lg border border-transparent px-[1.2em] py-[0.6em] text-base font-medium font-inherit text-[#0f0f0f] bg-white transition-[border-color] duration-[0.25s] shadow-[0_2px_2px_rgba(0,0,0,0.2)] outline-none cursor-pointer hover:border-[#396cd8] active:border-[#396cd8] active:bg-[#e8e8e8] dark:text-white dark:bg-[#0f0f0f98] dark:active:bg-[#0f0f0f69] ml-[5px]"
        >
            Uninstall
        </button>
    </form>

    {#if operationStatus}
        <div class="mt-5 p-2.5 bg-[#e8f4f8] rounded-lg dark:bg-[#1a3a4a]">
            <p><strong>Status:</strong> {operationStatus}</p>
        </div>
    {/if}

    {#if operationStatus && (progress.percentage > 0 || progress.status)}
        <div
            class="mt-5 p-[15px] bg-[#f0f0f0] rounded-lg max-w-[500px] mx-auto dark:bg-[#1a1a1a]"
        >
            <p><strong>Progress:</strong> {progress.percentage}%</p>
            <div
                class="w-full h-5 bg-[#ddd] rounded-[10px] overflow-hidden my-2.5 dark:bg-[#333]"
            >
                <div
                    class="h-full bg-[#396cd8] transition-[width] duration-300 ease-in-out"
                    style="width: {progress.percentage || 1}%"
                ></div>
            </div>
            <div class="flex justify-between items-center mt-[5px]">
                {#if progress.status}
                    <p class="text-[0.9em] text-[#666] m-0 dark:text-[#aaa]">
                        {progress.status}
                    </p>
                {/if}
                {#if progress.speed_mbps > 0}
                    <p
                        class="text-[0.9em] text-[#396cd8] font-semibold m-0 dark:text-[#24c8db]"
                    >
                        {progress.speed_mbps.toFixed(2)} MB/s
                    </p>
                {/if}
            </div>
        </div>
    {/if}
</main>
