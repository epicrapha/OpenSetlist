<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, createSetlist, listSetlists } from '#lib/api';
	import type { SetlistList, SetlistStatus } from '#lib/api';
	import { session } from '#lib/session.svelte';
	import { canWrite } from '#lib/roles';
	import { formatServiceDate, itemCountLabel, setlistStatusLabels } from '#lib/setlists';
	import Alert from '#lib/components/Alert.svelte';
	import Button from '#lib/components/Button.svelte';
	import Select from '#lib/components/Select.svelte';
	import TextField from '#lib/components/TextField.svelte';

	const statusStyles: Record<SetlistStatus, string> = {
		draft: 'bg-surface-2 text-muted',
		ready: 'bg-success-tint text-success-text',
		archived: 'bg-canvas text-muted ring-1 ring-border'
	};

	let q = $state('');
	let status = $state<SetlistStatus | ''>('');
	let result = $state<SetlistList | null>(null);
	let loading = $state(false);
	let error = $state('');
	let controller: AbortController | null = null;

	let title = $state('');
	let serviceDate = $state('');
	let serviceType = $state('');
	let creating = $state(false);
	let createError = $state('');
	let createErrorBox = $state<HTMLElement>();

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);
	const canEdit = $derived(canWrite(activeWorkspace?.role));
	const hasFilters = $derived(Boolean(q.trim() || status));

	async function load() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		controller?.abort();
		const current = new AbortController();
		controller = current;
		loading = true;
		error = '';
		try {
			const data = await listSetlists(
				workspaceId,
				{ q, status, limit: 100, offset: 0 },
				current.signal
			);
			if (current.signal.aborted) return;
			result = data;
		} catch (cause) {
			if (current.signal.aborted) return;
			error = cause instanceof ApiError ? cause.message : 'Could not load setlists.';
		} finally {
			if (!current.signal.aborted) loading = false;
		}
	}

	$effect(() => {
		void session.activeWorkspaceId;
		void q;
		void status;
		const timer = setTimeout(load, 220);
		return () => clearTimeout(timer);
	});

	$effect(() => {
		if (createError) createErrorBox?.focus();
	});

	async function create(event: SubmitEvent) {
		event.preventDefault();
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !canEdit) return;
		const cleanTitle = title.trim();
		if (!cleanTitle) {
			createError = 'Give the setlist a title before creating it.';
			return;
		}
		creating = true;
		createError = '';
		try {
			const created = await createSetlist(workspaceId, {
				title: cleanTitle,
				service_date: serviceDate || null,
				service_type: serviceType.trim() || null
			});
			await goto(resolve('/app/setlists/[id]', { id: created.id }));
		} catch (cause) {
			createError = cause instanceof ApiError ? cause.message : 'Could not create the setlist.';
		} finally {
			creating = false;
		}
	}

	function clearFilters() {
		q = '';
		status = '';
	}
</script>

{#if !session.activeWorkspaceId}
	<div class="max-w-lg">
		<h1 class="font-display text-display text-ink">Setlists</h1>
		<p class="mt-2 text-muted">Choose a workspace before you build a setlist.</p>
		<a
			href={resolve('/app')}
			class="mt-6 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
		>
			Choose a workspace
		</a>
	</div>
{:else}
	<header class="mb-8">
		<p class="text-sm font-medium tracking-[0.16em] text-accent uppercase">Run sheet</p>
		<h1 class="mt-1 font-display text-display text-ink">Setlists</h1>
		<p class="mt-2 max-w-prose text-muted">
			Plan the service from first song to last. Open one to add songs, prayers, and readings.
		</p>
	</header>

	{#if !canEdit}
		<Alert variant="info" class="mb-6">
			Your role in this workspace is read-only. You can open setlists but not create or change them.
		</Alert>
	{/if}

	<div class="grid gap-8 lg:grid-cols-[minmax(0,1.55fr)_minmax(18rem,1fr)] lg:items-start">
		<section aria-labelledby="setlists-heading">
			<h2 id="setlists-heading" class="sr-only">Your setlists</h2>

			<form
				class="mb-5 flex flex-col gap-3 sm:flex-row sm:items-end"
				onsubmit={(e) => e.preventDefault()}
			>
				<TextField
					label="Search setlists"
					name="setlist-search"
					type="search"
					placeholder="Title"
					bind:value={q}
					class="flex-1"
				/>
				<Select label="Status" name="setlist-status" bind:value={status} class="sm:w-44">
					<option value="">All statuses</option>
					<option value="draft">Draft</option>
					<option value="ready">Ready</option>
					<option value="archived">Archived</option>
				</Select>
				{#if hasFilters}
					<Button variant="ghost" onclick={clearFilters}>Clear</Button>
				{/if}
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
					Loading setlists…
				</div>
			{:else if result && result.setlists.length === 0 && !hasFilters}
				<div class="rounded-card border border-dashed border-border px-6 py-14 text-center">
					<span
						class="mx-auto grid size-12 place-items-center rounded-full bg-surface-2 text-muted"
						aria-hidden="true"
					>
						<svg viewBox="0 0 24 24" fill="none" class="size-6">
							<path
								d="M7 4h10a1 1 0 0 1 1 1v15l-6-3.5L6 20V5a1 1 0 0 1 1-1Z"
								stroke="currentColor"
								stroke-width="1.6"
								stroke-linejoin="round"
							/>
							<path
								d="M9 8h6M9 11h4"
								stroke="currentColor"
								stroke-width="1.6"
								stroke-linecap="round"
							/>
						</svg>
					</span>
					<h3 class="mt-4 font-display text-lg text-ink">No setlists yet</h3>
					<p class="mt-1 text-sm text-muted">
						{#if canEdit}
							Start with the service details on the right. You can add songs in the builder.
						{:else}
							Once a leader creates one, it will appear here.
						{/if}
					</p>
				</div>
			{:else if result && result.setlists.length === 0}
				<div class="rounded-card border border-border bg-surface px-6 py-12 text-center">
					<h3 class="font-display text-lg text-ink">No setlists match</h3>
					<p class="mt-1 text-sm text-muted">Try a different search or status.</p>
					<Button class="mt-5" variant="secondary" onclick={clearFilters}>Clear filters</Button>
				</div>
			{:else if result}
				<div aria-live="polite" class="mb-3 flex items-center justify-between text-sm text-muted">
					<span>
						{#if error}
							<span class="text-danger-text">Could not refresh. Showing the last results.</span>
						{:else}
							{result.total} {result.total === 1 ? 'setlist' : 'setlists'}
						{/if}
					</span>
					{#if loading}
						<span
							class="size-3.5 animate-spin rounded-full border-2 border-current border-t-transparent"
							aria-hidden="true"
						></span>
					{/if}
				</div>

				<ul
					class="divide-y divide-border overflow-hidden rounded-card border border-border bg-surface {loading
						? 'opacity-70'
						: ''}"
					aria-busy={loading}
				>
					{#each result.setlists as setlist (setlist.id)}
						<li>
							<a
								href={resolve('/app/setlists/[id]', { id: setlist.id })}
								class="group flex flex-wrap items-center justify-between gap-x-6 gap-y-3 px-4 py-4 transition-colors hover:bg-surface-2 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-accent sm:px-5"
							>
								<span class="min-w-0">
									<span class="flex flex-wrap items-center gap-2">
										<span class="truncate font-medium text-ink">{setlist.title}</span>
										{#if setlist.is_template}
											<span class="rounded-full bg-accent-tint px-2 py-0.5 text-xs text-accent">
												Template
											</span>
										{/if}
									</span>
									<span class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted">
										<span>{formatServiceDate(setlist.service_date)}</span>
										{#if setlist.service_type}
											<span aria-hidden="true"> · </span>
											<span class="truncate">{setlist.service_type}</span>
										{/if}
									</span>
								</span>
								<span class="flex shrink-0 items-center gap-2.5">
									<span class="text-sm text-muted">{itemCountLabel(setlist)}</span>
									<span
										class="rounded-full px-2.5 py-1 text-xs font-semibold {statusStyles[
											setlist.status
										]}"
									>
										{setlistStatusLabels[setlist.status]}
									</span>
									<svg
										viewBox="0 0 24 24"
										fill="none"
										class="size-4 text-muted transition-transform group-hover:translate-x-0.5 group-hover:text-ink"
										aria-hidden="true"
									>
										<path
											d="m9 6 6 6-6 6"
											stroke="currentColor"
											stroke-width="1.8"
											stroke-linecap="round"
											stroke-linejoin="round"
										/>
									</svg>
								</span>
							</a>
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		{#if canEdit}
			<section
				aria-labelledby="create-setlist-heading"
				class="rounded-card border border-border bg-surface p-6"
			>
				<h2 id="create-setlist-heading" class="font-display text-xl text-ink">New setlist</h2>
				<p class="mt-1 text-sm text-muted">
					Set the service once; the builder takes you from there.
				</p>

				<form class="mt-5 flex flex-col gap-4" onsubmit={create}>
					<TextField
						label="Title"
						name="setlist-title"
						placeholder="Sunday morning"
						required
						bind:value={title}
					/>
					<TextField
						label="Service date"
						name="setlist-date"
						type="date"
						bind:value={serviceDate}
					/>
					<TextField
						label="Service type"
						name="setlist-type"
						placeholder="Sunday morning"
						hint="Optional — e.g. Sunday evening, Midweek, Easter."
						list="service-types"
						bind:value={serviceType}
					/>
					<datalist id="service-types">
						<option value="Sunday morning"></option>
						<option value="Sunday evening"></option>
						<option value="Midweek"></option>
						<option value="Special event"></option>
					</datalist>

					{#if createError}
						<div bind:this={createErrorBox} tabindex="-1" class="rounded-field outline-none">
							<Alert>{createError}</Alert>
						</div>
					{/if}

					<Button type="submit" block loading={creating}>
						{creating ? 'Creating…' : 'Create setlist'}
					</Button>
				</form>
			</section>
		{/if}
	</div>
{/if}
