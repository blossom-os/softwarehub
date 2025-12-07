<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		value = 0,
		max = 100,
		class: className,
		...restProps
	}: HTMLAttributes<HTMLDivElement> & { value?: number; max?: number } = $props();

	let percentage = $derived(Math.min(Math.max((value / max) * 100, 0), 100));
</script>

<div
	class={cn("relative h-4 w-full overflow-hidden rounded-full bg-secondary", className)}
	role="progressbar"
	aria-valuemin={0}
	aria-valuemax={max}
	aria-valuenow={value}
	{...restProps}
>
	<div
		class="h-full w-full flex-1 bg-primary transition-all"
		style="transform: translateX(-{100 - percentage}%)"
	></div>
</div>

