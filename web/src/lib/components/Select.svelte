<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLSelectAttributes } from 'svelte/elements';

	const uid = $props.id();

	let {
		label,
		value = $bindable(''),
		hint = '',
		error = '',
		class: className = '',
		id = uid,
		children,
		...rest
	}: HTMLSelectAttributes & {
		label: string;
		value?: string;
		hint?: string;
		error?: string;
		class?: string;
		children: Snippet;
	} = $props();

	const hintId = $derived(`${id}-hint`);
	const errorId = $derived(`${id}-error`);
	const describedBy = $derived(
		[error ? errorId : '', hint ? hintId : ''].filter(Boolean).join(' ') || undefined
	);
</script>

<div class="flex flex-col gap-1.5 {className}">
	<label for={id} class="text-sm font-medium text-ink">{label}</label>
	<select
		{id}
		bind:value
		aria-invalid={error ? 'true' : undefined}
		aria-describedby={describedBy}
		class="min-h-11 w-full rounded-field bg-surface px-3.5 py-2 text-base text-ink ring-1 ring-border transition-colors focus:ring-2 focus:ring-accent focus:outline-none"
		{...rest}
	>
		{@render children()}
	</select>
	{#if error}
		<p id={errorId} class="text-sm text-danger-text">{error}</p>
	{:else if hint}
		<p id={hintId} class="text-sm text-muted">{hint}</p>
	{/if}
</div>
