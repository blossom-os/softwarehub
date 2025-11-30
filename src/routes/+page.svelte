<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount, onDestroy } from "svelte";

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

<main class="container">
    <h1>Software Hub</h1>

    <form class="row" onsubmit={install}>
        <input
            id="greet-input"
            placeholder="Enter Flatpak ref (e.g., org.example.App)..."
            bind:value={refName}
        />
        <button type="submit">Install</button>
        <button type="button" onclick={uninstall}>Uninstall</button>
    </form>

    {#if operationStatus}
        <div class="status">
            <p><strong>Status:</strong> {operationStatus}</p>
        </div>
    {/if}

    {#if operationStatus && (progress.percentage > 0 || progress.status)}
        <div class="progress-container">
            <p><strong>Progress:</strong> {progress.percentage}%</p>
            <div class="progress-bar">
                <div
                    class="progress-fill"
                    style="width: {progress.percentage || 1}%"
                ></div>
            </div>
            <div class="progress-info">
                {#if progress.status}
                    <p class="progress-status">{progress.status}</p>
                {/if}
                {#if progress.speed_mbps > 0}
                    <p class="progress-speed">
                        {progress.speed_mbps.toFixed(2)} MB/s
                    </p>
                {/if}
            </div>
        </div>
    {/if}
</main>

<style>
    .logo.vite:hover {
        filter: drop-shadow(0 0 2em #747bff);
    }

    .logo.svelte-kit:hover {
        filter: drop-shadow(0 0 2em #ff3e00);
    }

    :root {
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 24px;
        font-weight: 400;

        color: #0f0f0f;
        background-color: #f6f6f6;

        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        -webkit-text-size-adjust: 100%;
    }

    .container {
        margin: 0;
        padding-top: 10vh;
        display: flex;
        flex-direction: column;
        justify-content: center;
        text-align: center;
    }

    .logo {
        height: 6em;
        padding: 1.5em;
        will-change: filter;
        transition: 0.75s;
    }

    .logo.tauri:hover {
        filter: drop-shadow(0 0 2em #24c8db);
    }

    .row {
        display: flex;
        justify-content: center;
    }

    a {
        font-weight: 500;
        color: #646cff;
        text-decoration: inherit;
    }

    a:hover {
        color: #535bf2;
    }

    h1 {
        text-align: center;
    }

    input,
    button {
        border-radius: 8px;
        border: 1px solid transparent;
        padding: 0.6em 1.2em;
        font-size: 1em;
        font-weight: 500;
        font-family: inherit;
        color: #0f0f0f;
        background-color: #ffffff;
        transition: border-color 0.25s;
        box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    }

    button {
        cursor: pointer;
    }

    button:hover {
        border-color: #396cd8;
    }
    button:active {
        border-color: #396cd8;
        background-color: #e8e8e8;
    }

    input,
    button {
        outline: none;
    }

    #greet-input {
        margin-right: 5px;
    }

    button + button {
        margin-left: 5px;
    }

    .status {
        margin-top: 20px;
        padding: 10px;
        background-color: #e8f4f8;
        border-radius: 8px;
    }

    .progress-container {
        margin-top: 20px;
        padding: 15px;
        background-color: #f0f0f0;
        border-radius: 8px;
        max-width: 500px;
        margin-left: auto;
        margin-right: auto;
    }

    .progress-bar {
        width: 100%;
        height: 20px;
        background-color: #ddd;
        border-radius: 10px;
        overflow: hidden;
        margin: 10px 0;
    }

    .progress-fill {
        height: 100%;
        background-color: #396cd8;
        transition: width 0.3s ease;
    }

    .progress-info {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: 5px;
    }

    .progress-status {
        font-size: 0.9em;
        color: #666;
        margin: 0;
    }

    .progress-speed {
        font-size: 0.9em;
        color: #396cd8;
        font-weight: 600;
        margin: 0;
    }

    @media (prefers-color-scheme: dark) {
        .status {
            background-color: #1a3a4a;
        }

        .progress-container {
            background-color: #1a1a1a;
        }

        .progress-bar {
            background-color: #333;
        }

        .progress-status {
            color: #aaa;
        }

        .progress-speed {
            color: #24c8db;
        }
    }

    @media (prefers-color-scheme: dark) {
        :root {
            color: #f6f6f6;
            background-color: #2f2f2f;
        }

        a:hover {
            color: #24c8db;
        }

        input,
        button {
            color: #ffffff;
            background-color: #0f0f0f98;
        }
        button:active {
            background-color: #0f0f0f69;
        }
    }
</style>
