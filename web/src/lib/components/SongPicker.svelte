<script lang="ts">
	import { ApiError, listSongs } from '#lib/api';
	import type { Song } from '#lib/api';
	import Alert from './Alert.svelte';
	import Button from './Button.svelte';
	import TextField from './TextField.svelte';

	let {
		workspaceId,
		disabled = false,
		addSong
	}: {
		workspaceId: string;
		disabled?: boolean;
		addSong: (song: Song) => Promise<void>;
	} = $props();

	let query = $state('');
	let songs = $state<Song[]>([]);
	let loading = $state(false);
	let error = $state('');
	let addingId = $state<string | null>(null);
	let lastAdded = $state('');
	let controller: AbortController | null = null;

	const trimmed = $derived(query.trim());
	const resultLabel = $derived(
		loading
			? 'Searching…'
			: trimmed
				? `${songs.length} ${songs.length === 1 ? 'match' : 'matches'}`
				: `Suggested songs`
	);

	$effect(() => {
		if (disabled) return;
		void workspaceId;
		void trimmed;
		controller?.abort();
		const current = new AbortController();
		controller = current;
		loading = true;
		error = '';

		const timer = setTimeout(async () => {
			try {
				const data = await listSongs(
					workspaceId,
					{ q: trimmed, sort: 'title', limit: 8 },
					current.signal
				);
				if (current.signal.aborted) return;
				songs = data.songs;
			} catch (cause) {
				if (current.signal.aborted) return;
				error = cause instanceof ApiError ? cause.message : 'Could not search songs.';
				songs = [];
			} finally {
				if (!current.signal.aborted) loading = false;
			}
		}, 220);

		return () => {
			clearTimeout(timer);
			current.abort();
		};
	});

	async function choose(song: Song) {
		lastAdded = '';
		error = '';
		addingId = song.id;
		try {
			await addSong(song);
			lastAdded = song.title;
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not add that song.';
		} finally {
			addingId = null;
		}
	}
</script>

<section
	aria-labelledby="song-picker-heading"
	class="rounded-card border border-border bg-surface p-5"
>
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<h2 id="song-picker-heading" class="font-display text-lg text-ink">Add a song</h2>
			<p class="mt-0.5 text-sm text-muted">Search the library and quick-add it to the end.</p>
		</div>
	</div>

	<form class="mt-4" onsubmit={(event) => event.preventDefault()}>
		<TextField
			label="Search songs"
			name="song-search"
			type="search"
			placeholder="Title or artist"
			bind:value={query}
			{disabled}
			autocomplete="off"
		/>
	</form>

	<div class="mt-3 flex items-center justify-between gap-3 text-xs text-muted">
		<span aria-live="polite">{resultLabel}</span>
		{#if loading}
			<span
				class="size-3.5 animate-spin rounded-full border-2 border-current border-t-transparent"
				aria-hidden="true"
			></span>
		{/if}
	</div>

	{#if error}
		<div class="mt-3"><Alert>{error}</Alert></div>
	{/if}

	{#if songs.length > 0}
		<ul class="mt-2 divide-y divide-border overflow-hidden rounded-field border border-border">
			{#each songs as song (song.id)}
				<li class="flex items-center justify-between gap-3 bg-canvas/40 px-3 py-2.5">
					<span class="min-w-0">
						<span class="block truncate text-sm font-medium text-ink">{song.title}</span>
						<span class="block truncate text-xs text-muted">
							{song.artist ?? 'Unknown artist'}{#if song.default_key}
								<span aria-hidden="true"> · </span>Key
								<span class="font-mono font-semibold text-accent">{song.default_key}</span>
							{/if}
						</span>
					</span>
					<Button
						variant="secondary"
						disabled={disabled || addingId !== null}
						loading={addingId === song.id}
						aria-label="Add {song.title} to the setlist"
						onclick={() => choose(song)}
					>
						Add
					</Button>
				</li>
			{/each}
		</ul>
	{:else if !loading && !error}
		<p class="mt-3 text-sm text-muted">
			{trimmed ? `No songs match “${trimmed}”.` : 'No songs in this workspace yet.'}
		</p>
	{/if}

	<p role="status" class="sr-only">{lastAdded ? `${lastAdded} added.` : ''}</p>
</section>
