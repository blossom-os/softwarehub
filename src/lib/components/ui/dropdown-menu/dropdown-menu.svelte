<script lang="ts">
	import { cn } from "$lib/utils";
	
	type Props = {
		class?: string;
		children: any;
	};
	
	let { class: className = "", children }: Props = $props();
</script>

<div class={cn("relative inline-block", className)}>
	{@render children()}
</div>

