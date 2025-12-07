<script lang="ts">
	interface Props {
		progress: {
			stage: string;
			progress: number;
			total: number;
			percentage?: number;
			message: string;
			details?: string;
			cachedAppsCount?: number;
		} | null;
		visible?: boolean;
	}
	
	let { progress, visible = false }: Props = $props();
</script>

{#if visible && progress}
	<div class="w-full bg-gray-100 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 px-4 py-2">
		<div class="flex items-center justify-between mb-1">
			<div class="flex-1">
				<div class="text-sm font-medium text-gray-900 dark:text-gray-100">
					{progress.message}
				</div>
				{#if progress.details}
					<div class="text-xs text-gray-600 dark:text-gray-400 mt-0.5">
						{progress.details}
					</div>
				{/if}
			</div>
			<div class="ml-4 text-right">
				{#if progress.cachedAppsCount !== undefined && progress.cachedAppsCount > 0}
					<div class="text-sm font-medium text-gray-900 dark:text-gray-100">
						{progress.cachedAppsCount} apps cached
					</div>
				{/if}
				{#if progress.total > 0}
					<div class="text-sm font-medium text-gray-900 dark:text-gray-100">
						{progress.progress} / {progress.total}
					</div>
				{/if}
				{#if progress.percentage !== undefined}
					<div class="text-xs text-gray-600 dark:text-gray-400">
						{progress.percentage}%
					</div>
				{/if}
			</div>
		</div>
		<div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5 mt-1">
			<div 
				class="bg-blue-600 h-1.5 rounded-full transition-all duration-300"
				style="width: {progress.total > 0 ? (progress.progress / progress.total * 100) : 0}%"
			></div>
		</div>
	</div>
{/if}

