<script lang="ts">
	import Button from './Button.svelte';
	import TextField from './TextField.svelte';
	import Textarea from './Textarea.svelte';

	let {
		initialName = '',
		initialKey = '',
		initialChordpro = '',
		submitLabel = 'Save',
		saving = false,
		onSubmit,
		onCancel
	}: {
		initialName?: string;
		initialKey?: string;
		initialChordpro?: string;
		submitLabel?: string;
		saving?: boolean;
		onSubmit: (value: { name: string; key: string; chordpro: string }) => void;
		onCancel?: () => void;
	} = $props();

	// These are intentional one-time seeds; the form is remounted to reset.
	// svelte-ignore state_referenced_locally
	let name = $state(initialName);
	// svelte-ignore state_referenced_locally
	let key = $state(initialKey);
	// svelte-ignore state_referenced_locally
	let chordpro = $state(initialChordpro);

	function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!name.trim()) return;
		onSubmit({ name: name.trim(), key: key.trim(), chordpro });
	}
</script>

<form class="flex flex-col gap-4" onsubmit={submit}>
	<div class="grid gap-4 sm:grid-cols-2">
		<TextField label="Name" name="arrangement-name" required bind:value={name} />
		<TextField
			label="Key"
			name="arrangement-key"
			placeholder="G"
			hint="Optional"
			bind:value={key}
		/>
	</div>
	<Textarea
		label="ChordPro"
		name="arrangement-chordpro"
		rows={8}
		hint="Leave as is to reuse the current chart."
		bind:value={chordpro}
	/>
	<div class="flex flex-wrap gap-3">
		<Button type="submit" loading={saving}>{submitLabel}</Button>
		{#if onCancel}
			<Button type="button" variant="ghost" onclick={onCancel}>Cancel</Button>
		{/if}
	</div>
</form>
