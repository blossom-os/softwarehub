<script lang="ts">
	import { cn } from "$lib/utils";
	
	type Props = {
		class?: string;
		children: any;
		open?: boolean;
	};
	
	let { class: className = "", children, open = $bindable(false) }: Props = $props();
	let contentElement: HTMLDivElement | undefined;
	
	$effect(() => {
		if (!open) return;
		
		function handleClickOutside(event: MouseEvent) {
			if (contentElement && !contentElement.contains(event.target as Node)) {
				open = false;
			}
		}
		
		setTimeout(() => {
			document.addEventListener("click", handleClickOutside);
		}, 0);
		
		return () => {
			document.removeEventListener("click", handleClickOutside);
		};
	});
</script>

{#if open}
	<div
		bind:this={contentElement}
		class={cn(
			"absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 z-50",
			className
		)}
		role="menu"
		aria-orientation="vertical"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="py-1">
			{@render children()}
		</div>
	</div>
{/if}

