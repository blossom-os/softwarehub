<script lang="ts">
	import { cn } from "$lib/utils";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		orientation = "horizontal",
		class: className,
		...restProps
	}: HTMLAttributes<HTMLDivElement> & { orientation?: "horizontal" | "vertical" } = $props();
</script>

<div
	class={cn(
		"shrink-0 bg-border",
		orientation === "horizontal" ? "h-[1px] w-full" : "h-full w-[1px]",
		className
	)}
	{...restProps}
></div>

