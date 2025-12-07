<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onDestroy } from "svelte";
	import Progress from "$lib/components/ui/progress/progress.svelte";
	import Card from "$lib/components/ui/card/card.svelte";
	import CardContent from "$lib/components/ui/card/card-content.svelte";
	import CardHeader from "$lib/components/ui/card/card-header.svelte";
	import CardTitle from "$lib/components/ui/card/card-title.svelte";

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
		});

		unlistenOperationStarted = await listen("flatpak-operation-started", (event) => {
			const data = event.payload as {
				operation_type: string;
				ref: string;
			};
			operationStatus = `${data.operation_type} on ${data.ref}`;
		});

		unlistenComplete = await listen("flatpak-install-complete", (event) => {
			const data = event.payload as { ref: string };
			operationStatus = `Install complete: ${data.ref}`;
		});

		unlistenUninstallStarted = await listen("flatpak-uninstall-started", (event) => {
			const data = event.payload as { ref: string };
			progress = {
				percentage: 0,
				status: "Starting uninstallation...",
				ref: data.ref,
				speed_mbps: 0,
			};
			operationStatus = `Uninstalling: ${data.ref}`;
		});

		unlistenUninstallComplete = await listen("flatpak-uninstall-complete", (event) => {
			const data = event.payload as { ref: string };
			operationStatus = `Uninstall complete: ${data.ref}`;
		});
	});

	onDestroy(() => {
		unlistenProgress?.();
		unlistenOperationStarted?.();
		unlistenComplete?.();
		unlistenUninstallStarted?.();
		unlistenUninstallComplete?.();
	});
</script>

{#if operationStatus || progress.percentage > 0}
	<Card class="fixed bottom-4 right-4 w-96 z-50 shadow-lg">
		<CardHeader>
			<CardTitle>Download Status</CardTitle>
		</CardHeader>
		<CardContent>
			{#if operationStatus}
				<p class="text-sm mb-2">{operationStatus}</p>
			{/if}
			{#if progress.percentage > 0}
				<Progress value={progress.percentage} class="mb-2" />
				<div class="flex justify-between text-xs text-muted-foreground">
					<span>{progress.status}</span>
					{#if progress.speed_mbps > 0}
						<span>{progress.speed_mbps.toFixed(2)} MB/s</span>
					{/if}
				</div>
			{/if}
		</CardContent>
	</Card>
{/if}

