<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { buttonClasses, type ButtonSize, type ButtonVariant } from './button';

	let {
		variant = 'primary',
		size = 'md',
		loading = false,
		block = false,
		type = 'button',
		disabled = false,
		class: className = '',
		children,
		...rest
	}: HTMLButtonAttributes & {
		variant?: ButtonVariant;
		size?: ButtonSize;
		loading?: boolean;
		block?: boolean;
		class?: string;
		children: Snippet;
	} = $props();
</script>

<button
	{type}
	disabled={disabled || loading}
	aria-busy={loading}
	class={buttonClasses(variant, size, block, className)}
	{...rest}
>
	{#if loading}
		<span
			class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
			aria-hidden="true"
		></span>
	{/if}
	{@render children()}
</button>
