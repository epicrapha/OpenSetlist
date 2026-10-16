<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'error' | 'info' | 'success';

	let {
		variant = 'error',
		children,
		class: className = ''
	}: { variant?: Variant; children: Snippet; class?: string } = $props();

	const styles: Record<Variant, string> = {
		error: 'bg-danger-tint text-danger-text ring-danger/25',
		success: 'bg-success-tint text-success-text ring-success/25',
		info: 'bg-surface-2 text-ink ring-border'
	};
	const role = $derived(variant === 'error' ? 'alert' : 'status');
</script>

<p {role} class="rounded-field px-3.5 py-2.5 text-sm ring-1 {styles[variant]} {className}">
	{@render children()}
</p>
