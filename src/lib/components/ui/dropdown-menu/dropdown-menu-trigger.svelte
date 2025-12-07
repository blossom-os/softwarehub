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
	class={cn("outline-none", className)}
	onclick={onclick}
	onkeydown={(e) => {
		if (e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			onclick?.();
		}
	}}
>
	{@render children()}
</button>

