<script lang="ts">
	import type { HTMLTextareaAttributes } from 'svelte/elements';

	const uid = $props.id();

	let {
		label,
		value = $bindable(''),
		hint = '',
		error = '',
		class: className = '',
		id = uid,
		...rest
	}: HTMLTextareaAttributes & {
		label: string;
		value?: string;
		hint?: string;
		error?: string;
		class?: string;
	} = $props();

	const hintId = $derived(`${id}-hint`);
	const errorId = $derived(`${id}-error`);
	const describedBy = $derived(
		[error ? errorId : '', hint ? hintId : ''].filter(Boolean).join(' ') || undefined
	);
</script>

<div class="flex flex-col gap-1.5 {className}">
	<label for={id} class="text-sm font-medium text-ink">{label}</label>
	<textarea
		{id}
		bind:value
		aria-invalid={error ? 'true' : undefined}
		aria-describedby={describedBy}
		class="w-full rounded-field bg-surface px-3.5 py-2 font-mono text-sm leading-relaxed text-ink ring-1 ring-border transition-colors placeholder:text-muted/60 focus:ring-2 focus:ring-accent focus:outline-none"
		{...rest}></textarea>
	{#if error}
		<p id={errorId} class="text-sm text-danger-text">{error}</p>
	{:else if hint}
		<p id={hintId} class="text-sm text-muted">{hint}</p>
	{/if}
</div>
