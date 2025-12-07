<script lang="ts">
	import { cn } from "$lib/utils";
	
	type Props = {
		class?: string;
		children: any;
		onclick?: () => void;
	};
	
	let { class: className = "", children, onclick }: Props = $props();
</script>

<button
	type="button"
	class={cn(
		"block w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer",
		className
	)}
	onclick={onclick}
	role="menuitem"
>
	{@render children()}
</button>

