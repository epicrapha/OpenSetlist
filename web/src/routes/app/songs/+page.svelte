<script lang="ts">
	import { resolve } from '$app/paths';
	import { ApiError, listSongs } from '#lib/api';
	import type { SongList, SongSort } from '#lib/api';
	import { session } from '#lib/session.svelte';
	import { canWrite } from '#lib/roles';
	import Alert from '#lib/components/Alert.svelte';
	import Button from '#lib/components/Button.svelte';
	import Select from '#lib/components/Select.svelte';
	import TextField from '#lib/components/TextField.svelte';

	const PAGE_SIZE = 20;

	let q = $state('');
	let key = $state('');
	let artist = $state('');
	let tag = $state('');
	let sort = $state<SongSort>('title');
	let offset = $state(0);

	let loading = $state(false);
	let error = $state('');
	let result = $state<SongList | null>(null);
	let controller: AbortController | null = null;

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);
	const canEdit = $derived(canWrite(activeWorkspace?.role));
	const hasFilters = $derived(Boolean(q.trim() || key.trim() || artist.trim() || tag.trim()));

	const total = $derived(result?.total ?? 0);
	const pageStart = $derived(total === 0 ? 0 : offset + 1);
	const pageEnd = $derived(Math.min(offset + PAGE_SIZE, total));
	const canPrev = $derived(offset > 0);
	const canNext = $derived(offset + PAGE_SIZE < total);

	async function load() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		controller?.abort();
		const current = new AbortController();
		controller = current;
		loading = true;
		error = '';
		try {
			const data = await listSongs(
				workspaceId,
				{ q, key, artist, tag, sort, limit: PAGE_SIZE, offset },
				current.signal
			);
			if (current.signal.aborted) return;
			result = data;
		} catch (cause) {
			if (current.signal.aborted) return;
			error = cause instanceof ApiError ? cause.message : 'Could not load songs.';
		} finally {
			if (!current.signal.aborted) loading = false;
		}
	}

	$effect(() => {
		// Track every input that should trigger a reload, then debounce the burst
		// of keystrokes into one request.
		const dependencies = [q, key, artist, tag, sort, offset, session.activeWorkspaceId];
		void dependencies;
		const timer = setTimeout(load, 220);
		return () => clearTimeout(timer);
	});

	function onFilterInput() {
		offset = 0;
	}

	function clearFilters() {
		q = '';
		key = '';
		artist = '';
		tag = '';
		offset = 0;
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString(undefined, {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		});
	}
</script>

{#if !session.activeWorkspaceId}
	<div class="max-w-lg">
		<h1 class="font-display text-display text-ink">Songs</h1>
		<p class="mt-2 text-muted">Choose a workspace before you browse songs.</p>
		<a
			href={resolve('/app')}
			class="mt-6 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
		>
			Choose a workspace
		</a>
	</div>
{:else}
	<header class="mb-8 flex flex-wrap items-end justify-between gap-4">
		<div>
			<h1 class="font-display text-display text-ink">Songs</h1>
			<p class="mt-2 max-w-prose text-muted">
				Every song in {activeWorkspace?.name ?? 'this workspace'}, with its key, tempo, and chords.
			</p>
		</div>
		{#if canEdit}
			<a
				href={resolve('/app/songs/new')}
				class="inline-flex min-h-11 items-center gap-2 rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
			>
				<svg viewBox="0 0 24 24" fill="none" class="size-4" aria-hidden="true">
					<path
						d="M12 5v14M5 12h14"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
					/>
				</svg>
				New song
			</a>
		{/if}
	</header>

	{#if !canEdit}
		<Alert variant="info" class="mb-6">
			Your role in this workspace is read-only. You can browse songs but not change them.
		</Alert>
	{/if}

	<form class="mb-8 flex flex-col gap-4" onsubmit={(event) => event.preventDefault()}>
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
			<TextField
				label="Search"
				name="q"
				type="search"
				placeholder="Title or artist"
				bind:value={q}
				oninput={onFilterInput}
			/>
			<TextField
				label="Key"
				name="key"
				placeholder="G"
				hint="Exact key"
				bind:value={key}
				oninput={onFilterInput}
			/>
			<TextField
				label="Artist"
				name="artist"
				placeholder="Matt Redman"
				bind:value={artist}
				oninput={onFilterInput}
			/>
			<TextField
				label="Tag"
				name="tag"
				placeholder="easter"
				bind:value={tag}
				oninput={onFilterInput}
			/>
		</div>
		<div class="flex flex-wrap items-end gap-4">
			<Select label="Sort by" name="sort" bind:value={sort} class="w-48">
				<option value="title">Title</option>
				<option value="updated_at">Recently updated</option>
			</Select>
			{#if hasFilters}
				<Button variant="ghost" onclick={clearFilters}>Clear filters</Button>
			{/if}
		</div>
	</form>

	{#if error && !result}
		<div class="max-w-lg">
			<Alert>{error}</Alert>
			<Button class="mt-4" variant="secondary" onclick={load}>Try again</Button>
		</div>
	{:else if !result && loading}
		<div role="status" class="flex items-center gap-3 py-10 text-muted">
			<span
				class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
				aria-hidden="true"
			></span>
			Loading songs…
		</div>
	{:else if result && total === 0 && !hasFilters}
		<div class="rounded-card border border-dashed border-border px-6 py-14 text-center">
			<span
				class="mx-auto grid size-12 place-items-center rounded-full bg-surface-2 text-muted"
				aria-hidden="true"
			>
				<svg viewBox="0 0 24 24" fill="none" class="size-6">
					<path
						d="M9 18V6l10-2v12"
						stroke="currentColor"
						stroke-width="1.6"
						stroke-linejoin="round"
					/>
					<circle cx="6.5" cy="18" r="2.5" stroke="currentColor" stroke-width="1.6" />
					<circle cx="16.5" cy="16" r="2.5" stroke="currentColor" stroke-width="1.6" />
				</svg>
			</span>
			<h2 class="mt-4 font-display text-lg text-ink">No songs yet</h2>
			<p class="mt-1 text-sm text-muted">Add your first song to start building setlists from it.</p>
			{#if canEdit}
				<a
					href={resolve('/app/songs/new')}
					class="mt-5 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
				>
					Add a song
				</a>
			{/if}
		</div>
	{:else if result && total === 0}
		<div class="rounded-card border border-border bg-surface px-6 py-12 text-center">
			<h2 class="font-display text-lg text-ink">No songs match these filters</h2>
			<p class="mt-1 text-sm text-muted">Try a different search, or clear the filters.</p>
			<Button class="mt-5" variant="secondary" onclick={clearFilters}>Clear filters</Button>
		</div>
	{:else if result}
		<div aria-live="polite" class="mb-3 text-sm text-muted">
			{#if error}
				<span class="text-danger-text">Could not refresh. Showing the last results.</span>
			{:else}
				Showing {pageStart}–{pageEnd} of {total}
			{/if}
		</div>
		<ul
			class="divide-y divide-border overflow-hidden rounded-card border border-border bg-surface {loading
				? 'opacity-60'
				: ''}"
			aria-busy={loading}
		>
			{#each result.songs as song (song.id)}
				<li>
					<a
						href={resolve('/app/songs/[id]', { id: song.id })}
						class="flex flex-wrap items-center justify-between gap-x-6 gap-y-2 px-4 py-3.5 transition-colors hover:bg-surface-2 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent sm:px-5"
					>
						<span class="flex min-w-0 flex-col">
							<span class="truncate font-medium text-ink">{song.title}</span>
							{#if song.artist}
								<span class="truncate text-sm text-muted">{song.artist}</span>
							{/if}
						</span>
						<span class="flex flex-wrap items-center gap-2">
							{#if song.tags.length > 0}
								<span class="hidden items-center gap-1.5 md:flex">
									{#each song.tags.slice(0, 3) as item (item)}
										<span class="rounded-full bg-surface-2 px-2.5 py-0.5 text-xs text-muted"
											>{item}</span
										>
									{/each}
								</span>
							{/if}
							<span class="hidden text-xs text-muted sm:block">{formatDate(song.updated_at)}</span>
							{#if song.default_key}
								<span
									class="inline-flex min-w-9 justify-center rounded-md bg-accent-tint px-2 py-1 font-mono text-sm font-semibold text-accent"
								>
									{song.default_key}
								</span>
							{/if}
						</span>
					</a>
				</li>
			{/each}
		</ul>

		{#if total > PAGE_SIZE}
			<div class="mt-5 flex items-center justify-between gap-4">
				<Button
					variant="secondary"
					disabled={!canPrev}
					onclick={() => (offset = Math.max(0, offset - PAGE_SIZE))}
				>
					Previous
				</Button>
				<span class="text-sm text-muted">Page {Math.floor(offset / PAGE_SIZE) + 1}</span>
				<Button
					variant="secondary"
					disabled={!canNext}
					onclick={() => (offset = offset + PAGE_SIZE)}
				>
					Next
				</Button>
			</div>
		{/if}
	{/if}
{/if}
