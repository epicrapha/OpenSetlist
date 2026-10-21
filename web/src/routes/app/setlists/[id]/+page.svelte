<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		addItem,
		deleteItem,
		deleteSetlist,
		duplicateSetlist,
		getSetlist,
		reorderItems,
		shareSetlist,
		unshareSetlist,
		updateItem,
		updateSetlist
	} from '#lib/api';
	import type {
		Setlist,
		SetlistItem,
		SetlistStatus,
		Song,
		UpdateSetlistInput,
		UpdateSetlistItemInput
	} from '#lib/api';
	import { session } from '#lib/session.svelte';
	import { canWrite } from '#lib/roles';
	import { copyText } from '#lib/clipboard';
	import {
		formatDuration,
		formatServiceDate,
		itemChart,
		itemDuration,
		itemKey,
		itemTempo,
		itemTitle,
		notePresets,
		runHints,
		setlistStatusLabels,
		totalDuration
	} from '#lib/setlists';
	import Alert from '#lib/components/Alert.svelte';
	import Button from '#lib/components/Button.svelte';
	import ChartView from '#lib/components/ChartView.svelte';
	import Select from '#lib/components/Select.svelte';
	import SongPicker from '#lib/components/SongPicker.svelte';
	import TextField from '#lib/components/TextField.svelte';
	import Textarea from '#lib/components/Textarea.svelte';
	import type { PageProps } from './$types';

	let { params }: PageProps = $props();

	const setlistId = $derived(params.id);

	let setlist = $state<Setlist | null>(null);
	let items = $state<SetlistItem[]>([]);
	let loading = $state(true);
	let error = $state('');
	let errorBox = $state<HTMLElement>();
	let notice = $state('');
	let noticeTimer: ReturnType<typeof setTimeout> | undefined;

	let headerDraft = $state({
		title: '',
		service_date: '',
		service_type: '',
		status: 'draft' as string
	});
	let savingHeader = $state(false);
	let duplicating = $state(false);
	let templating = $state(false);
	let deletingSetlist = $state(false);
	let confirmSetlistDelete = $state(false);

	let sharing = $state(false);
	let revoking = $state(false);
	let shareError = $state('');
	let shareErrorBox = $state<HTMLElement>();
	let copied = $state(false);
	let copyError = $state('');
	let confirmRevoke = $state(false);
	let copyTimer: ReturnType<typeof setTimeout> | undefined;

	let expanded = $state<Record<string, boolean>>({});
	let editingItem = $state<string | null>(null);
	let itemDraft = $state({
		title: '',
		key: '',
		capo: '0',
		notes: '',
		duration: ''
	});
	let itemError = $state('');
	let itemErrorBox = $state<HTMLElement>();
	let savingItem = $state<string | null>(null);
	let deletingItemId = $state<string | null>(null);
	let confirmItemDelete = $state<string | null>(null);

	let reordering = $state(false);
	let draggedId = $state<string | null>(null);
	let dragOverId = $state<string | null>(null);
	let dragAfter = $state(false);

	let noteType = $state<string>(notePresets[0]);
	let noteTitle = $state('');
	let noteNotes = $state('');
	let addingNote = $state(false);
	let noteError = $state('');
	let noteErrorBox = $state<HTMLElement>();

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);
	const canEdit = $derived(canWrite(activeWorkspace?.role));
	const canEditItems = $derived(canEdit && setlist?.status !== 'archived');
	const headerLocked = $derived(!canEdit || setlist?.status === 'archived');

	const hints = $derived(runHints(items));
	const total = $derived(totalDuration(items));
	const songCount = $derived(items.filter((item) => item.kind === 'song').length);
	const noteCount = $derived(items.length - songCount);
	const headerDirty = $derived(
		!!setlist &&
			(headerDraft.title.trim() !== setlist.title ||
				headerDraft.service_date !== (setlist.service_date ?? '') ||
				headerDraft.service_type.trim() !== (setlist.service_type ?? '') ||
				headerDraft.status !== setlist.status)
	);

	function syncHeader(value: Setlist) {
		headerDraft = {
			title: value.title,
			service_date: value.service_date ?? '',
			service_type: value.service_type ?? '',
			status: value.status
		};
	}

	function flash(message: string) {
		notice = message;
		clearTimeout(noticeTimer);
		noticeTimer = setTimeout(() => (notice = ''), 3200);
	}

	function fail(message: string) {
		error = message;
	}

	async function load() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) {
			loading = false;
			return;
		}
		loading = true;
		error = '';
		try {
			const detail = await getSetlist(workspaceId, setlistId);
			setlist = detail.setlist;
			items = detail.items;
			syncHeader(detail.setlist);
		} catch (cause) {
			setlist = null;
			items = [];
			error =
				cause instanceof ApiError && cause.status === 404
					? 'This setlist was not found in the current workspace.'
					: cause instanceof ApiError
						? cause.message
						: 'Could not load the setlist.';
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		void setlistId;
		void session.activeWorkspaceId;
		void load();
	});

	$effect(() => {
		if (error) errorBox?.focus();
	});

	$effect(() => {
		if (itemError) itemErrorBox?.focus();
	});

	$effect(() => {
		if (noteError) noteErrorBox?.focus();
	});

	$effect(() => {
		if (shareError || copyError) shareErrorBox?.focus();
	});

	function setItemCount(count: number) {
		if (setlist) setlist = { ...setlist, item_count: count };
	}

	async function saveHeader(event: SubmitEvent) {
		event.preventDefault();
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEdit) return;
		const title = headerDraft.title.trim();
		if (!title) {
			fail('The title must not be empty.');
			return;
		}

		const payload: UpdateSetlistInput = {};
		if (title !== setlist.title) payload.title = title;
		const serviceDate = headerDraft.service_date || null;
		if (serviceDate !== setlist.service_date) payload.service_date = serviceDate;
		const serviceType = headerDraft.service_type.trim() || null;
		if (serviceType !== setlist.service_type) payload.service_type = serviceType;
		if (headerDraft.status !== setlist.status) payload.status = headerDraft.status as SetlistStatus;
		if (Object.keys(payload).length === 0) return;

		savingHeader = true;
		error = '';
		notice = '';
		try {
			const updated = await updateSetlist(workspaceId, setlist.id, payload);
			setlist = updated;
			syncHeader(updated);
			flash('Setlist details saved.');
		} catch (cause) {
			fail(cause instanceof ApiError ? cause.message : 'Could not save the setlist.');
		} finally {
			savingHeader = false;
		}
	}

	async function duplicate(template: boolean) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEdit) return;
		if (template) templating = true;
		else duplicating = true;
		error = '';
		try {
			const copy = await duplicateSetlist(workspaceId, setlist.id);
			if (template) {
				const base = copy.title.replace(/\s*\(copy\)$/i, '');
				await updateSetlist(workspaceId, copy.id, {
					is_template: true,
					title: `${base} (template)`
				});
			}
			await goto(resolve('/app/setlists/[id]', { id: copy.id }));
		} catch (cause) {
			fail(cause instanceof ApiError ? cause.message : 'Could not duplicate the setlist.');
			duplicating = false;
			templating = false;
		}
	}

	async function removeSetlist() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEdit) return;
		deletingSetlist = true;
		error = '';
		try {
			await deleteSetlist(workspaceId, setlist.id);
			await goto(resolve('/app/setlists'));
		} catch (cause) {
			fail(cause instanceof ApiError ? cause.message : 'Could not delete the setlist.');
			confirmSetlistDelete = false;
			deletingSetlist = false;
		}
	}

	// ---------------------------------------------------------------------
	// Sharing
	// ---------------------------------------------------------------------

	function sharePath(token: string): string {
		return `/s/${token}`;
	}

	function shareUrl(token: string): string {
		return typeof window === 'undefined'
			? sharePath(token)
			: `${window.location.origin}${sharePath(token)}`;
	}

	async function copyShareLink(token: string) {
		copyError = '';
		const ok = await copyText(shareUrl(token));
		if (!ok) {
			copyError = 'Could not copy the link. Open it below and copy from the address bar.';
			return;
		}
		copied = true;
		clearTimeout(copyTimer);
		copyTimer = setTimeout(() => (copied = false), 2200);
	}

	async function createShare() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEditItems) return;
		sharing = true;
		shareError = '';
		try {
			const result = await shareSetlist(workspaceId, setlist.id);
			setlist = { ...setlist, share_token: result.share_token };
			await copyShareLink(result.share_token);
			flash('Share link created and copied.');
		} catch (cause) {
			shareError = cause instanceof ApiError ? cause.message : 'Could not create a share link.';
		} finally {
			sharing = false;
		}
	}

	async function revokeShare() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist) return;
		revoking = true;
		shareError = '';
		try {
			await unshareSetlist(workspaceId, setlist.id);
			setlist = { ...setlist, share_token: null };
			confirmRevoke = false;
			copied = false;
			flash('Share link revoked. The old link no longer works.');
		} catch (cause) {
			shareError = cause instanceof ApiError ? cause.message : 'Could not revoke the link.';
			confirmRevoke = false;
		} finally {
			revoking = false;
		}
	}

	// ---------------------------------------------------------------------
	// Items
	// ---------------------------------------------------------------------

	async function addSong(song: Song) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEditItems) {
			throw new ApiError(409, 'read_only', 'This setlist is read-only.');
		}
		const created = await addItem(workspaceId, setlist.id, {
			kind: 'song',
			song_id: song.id
		});
		items = [...items, created].sort((a, b) => a.position - b.position);
		setItemCount(items.length);
		flash(`${song.title} added.`);
	}

	function chooseNoteType(event: Event) {
		noteType = (event.currentTarget as HTMLSelectElement).value;
		if (noteType !== 'Other') noteTitle = noteType;
	}

	async function addNote(event: SubmitEvent) {
		event.preventDefault();
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEditItems) return;
		const title = noteTitle.trim() || (noteType === 'Other' ? '' : noteType);
		if (!title) {
			noteError = 'Give the item a title, for example “Prayer”.';
			return;
		}
		addingNote = true;
		noteError = '';
		try {
			const created = await addItem(workspaceId, setlist.id, {
				kind: 'note',
				title,
				notes: noteNotes.trim() || null
			});
			items = [...items, created].sort((a, b) => a.position - b.position);
			setItemCount(items.length);
			noteTitle = '';
			noteNotes = '';
			noteType = notePresets[0];
			flash(`${title} added.`);
		} catch (cause) {
			noteError = cause instanceof ApiError ? cause.message : 'Could not add that item.';
		} finally {
			addingNote = false;
		}
	}

	function toggleExpanded(item: SetlistItem) {
		expanded = { ...expanded, [item.id]: !expanded[item.id] };
	}

	function startEdit(item: SetlistItem) {
		editingItem = item.id;
		itemError = '';
		itemDraft = {
			title: item.title ?? '',
			key: item.key_override ?? '',
			capo: String(item.capo_override ?? 0),
			notes: item.notes ?? '',
			duration: item.duration_seconds === null ? '' : String(item.duration_seconds)
		};
		expanded = { ...expanded, [item.id]: true };
	}

	function replaceItem(updated: SetlistItem) {
		items = items.map((item) => (item.id === updated.id ? updated : item));
	}

	// `bind:value` on a number input hands back a number, despite the prop type.
	function parseDuration(value: string | number): number | null | undefined {
		const text = String(value).trim();
		if (!text) return null;
		const parsed = Number.parseInt(text, 10);
		if (!Number.isFinite(parsed) || parsed < 0) return undefined;
		return parsed;
	}

	async function saveItem(item: SetlistItem) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEditItems) return;
		const duration = parseDuration(itemDraft.duration);
		if (duration === undefined) {
			itemError = 'Duration must be a whole number of seconds, or blank to use the default.';
			return;
		}

		const payload: UpdateSetlistItemInput = {
			notes: itemDraft.notes.trim() || null,
			duration_seconds: duration
		};
		if (item.kind === 'note') {
			const title = itemDraft.title.trim();
			if (!title) {
				itemError = 'Give this item a title.';
				return;
			}
			payload.title = title;
		} else {
			payload.key_override = itemDraft.key.trim() || null;
			const capo = Number.parseInt(itemDraft.capo, 10);
			payload.capo_override = Number.isFinite(capo) && capo > 0 ? capo : null;
		}

		savingItem = item.id;
		itemError = '';
		error = '';
		try {
			const updated = await updateItem(workspaceId, setlist.id, item.id, payload);
			replaceItem(updated);
			editingItem = null;
			flash('Item updated.');
		} catch (cause) {
			itemError = cause instanceof ApiError ? cause.message : 'Could not update the item.';
		} finally {
			savingItem = null;
		}
	}

	async function resetItem(item: SetlistItem) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEditItems) return;
		savingItem = item.id;
		itemError = '';
		try {
			const updated = await updateItem(workspaceId, setlist.id, item.id, {
				key_override: null,
				capo_override: null,
				duration_seconds: null
			});
			replaceItem(updated);
			itemDraft = {
				...itemDraft,
				key: '',
				capo: '0',
				duration: ''
			};
			flash('Key, capo, and duration reset to the library defaults.');
		} catch (cause) {
			itemError = cause instanceof ApiError ? cause.message : 'Could not reset the item.';
		} finally {
			savingItem = null;
		}
	}

	async function removeItem(item: SetlistItem) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist || !canEditItems) return;
		deletingItemId = item.id;
		error = '';
		try {
			await deleteItem(workspaceId, setlist.id, item.id);
			items = items
				.filter((entry) => entry.id !== item.id)
				.map((entry, index) => ({ ...entry, position: index }));
			setItemCount(items.length);
			if (editingItem === item.id) editingItem = null;
			confirmItemDelete = null;
			flash(`${itemTitle(item)} removed.`);
		} catch (cause) {
			fail(cause instanceof ApiError ? cause.message : 'Could not remove the item.');
			confirmItemDelete = null;
		} finally {
			deletingItemId = null;
		}
	}

	function moveLocally(from: number, to: number) {
		const next = [...items];
		const [moved] = next.splice(from, 1);
		next.splice(to, 0, moved);
		items = next;
	}

	async function persistOrder(previous: SetlistItem[], next: SetlistItem[]) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !setlist) return;
		reordering = true;
		error = '';
		try {
			items = await reorderItems(
				workspaceId,
				setlist.id,
				next.map((item) => item.id)
			);
		} catch (cause) {
			items = previous;
			fail(cause instanceof ApiError ? cause.message : 'Could not save the new order.');
		} finally {
			reordering = false;
		}
	}

	function move(index: number, delta: number) {
		if (!canEditItems || reordering) return;
		const target = index + delta;
		if (target < 0 || target >= items.length) return;
		const previous = items;
		moveLocally(index, target);
		void persistOrder(previous, items);
	}

	function dragStart(event: DragEvent, item: SetlistItem) {
		if (!canEditItems) {
			event.preventDefault();
			return;
		}
		draggedId = item.id;
		dragOverId = null;
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
			event.dataTransfer.setData('text/plain', item.id);
		}
	}

	function dragOver(event: DragEvent, item: SetlistItem) {
		if (!draggedId || draggedId === item.id) return;
		event.preventDefault();
		if (event.dataTransfer) event.dataTransfer.dropEffect = 'move';
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		dragAfter = event.clientY > rect.top + rect.height / 2;
		dragOverId = item.id;
	}

	function drop(event: DragEvent, target: SetlistItem) {
		event.preventDefault();
		const source = draggedId || event.dataTransfer?.getData('text/plain') || '';
		const after = dragAfter;
		draggedId = null;
		dragOverId = null;
		if (!source || source === target.id || !canEditItems || reordering) return;

		const previous = items;
		const moved = items.find((item) => item.id === source);
		if (!moved) return;
		const next = items.filter((item) => item.id !== source);
		const targetIndex = next.findIndex((item) => item.id === target.id);
		if (targetIndex === -1) return;
		next.splice(targetIndex + (after ? 1 : 0), 0, moved);
		items = next;
		void persistOrder(previous, next);
	}

	function dragEnd() {
		draggedId = null;
		dragOverId = null;
	}
</script>

{#if loading}
	<div role="status" class="flex items-center gap-3 py-10 text-muted">
		<span
			class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
			aria-hidden="true"
		></span>
		Loading setlist…
	</div>
{:else if !setlist}
	<div class="max-w-lg">
		<h1 class="font-display text-display text-ink">Setlist not found</h1>
		<p class="mt-2 text-muted">{error || 'This setlist could not be loaded.'}</p>
		<a
			href={resolve('/app/setlists')}
			class="mt-6 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
		>
			Back to setlists
		</a>
	</div>
{:else}
	<nav aria-label="Breadcrumb" class="print-hide mb-6 text-sm text-muted">
		<a href={resolve('/app/setlists')} class="font-medium text-accent hover:underline">Setlists</a>
		<span class="mx-2" aria-hidden="true">/</span>
		<span class="text-ink">{setlist.title}</span>
	</nav>

	<header class="print-hide mb-8 flex flex-wrap items-start justify-between gap-5">
		<div class="min-w-0">
			<div class="flex flex-wrap items-center gap-2.5">
				<h1 class="font-display text-display text-ink">{setlist.title}</h1>
				<span
					class="rounded-full px-2.5 py-1 text-xs font-semibold {setlist.status === 'ready'
						? 'bg-success-tint text-success-text'
						: setlist.status === 'archived'
							? 'bg-canvas text-muted ring-1 ring-border'
							: 'bg-surface-2 text-muted'}"
				>
					{setlistStatusLabels[setlist.status]}
				</span>
				{#if setlist.is_template}
					<span class="rounded-full bg-accent-tint px-2.5 py-1 text-xs font-semibold text-accent">
						Template
					</span>
				{/if}
			</div>
			<p class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-muted">
				<span>{formatServiceDate(setlist.service_date)}</span>
				{#if setlist.service_type}
					<span aria-hidden="true"> · </span>
					<span>{setlist.service_type}</span>
				{/if}
				<span aria-hidden="true"> · </span>
				<span>{formatDuration(total)}</span>
			</p>
		</div>

		{#if canEdit}
			<div class="flex flex-wrap items-center gap-2">
				<Button variant="secondary" loading={duplicating} onclick={() => duplicate(false)}>
					Duplicate
				</Button>
				<Button variant="secondary" loading={templating} onclick={() => duplicate(true)}>
					Save as template
				</Button>
				{#if confirmSetlistDelete}
					<Button variant="danger" loading={deletingSetlist} onclick={removeSetlist}>
						Yes, delete
					</Button>
					<Button variant="ghost" onclick={() => (confirmSetlistDelete = false)}>Cancel</Button>
				{:else}
					<Button
						variant="ghost"
						class="text-danger-text hover:bg-danger-tint"
						onclick={() => (confirmSetlistDelete = true)}>Delete setlist</Button
					>
				{/if}
			</div>
		{/if}
	</header>

	{#if error}
		<div bind:this={errorBox} tabindex="-1" class="mb-6 rounded-field outline-none">
			<Alert>{error}</Alert>
		</div>
	{/if}
	{#if notice}
		<Alert variant="success" class="print-hide mb-6">{notice}</Alert>
	{/if}
	{#if setlist.status === 'archived'}
		<Alert variant="info" class="print-hide mb-6">
			This setlist is archived and read-only. Set the status to Draft or Ready below to edit it
			again.
		</Alert>
	{:else if !canEdit}
		<Alert variant="info" class="print-hide mb-6">
			Your role in this workspace is read-only. You can view and print this setlist, but not change
			it.
		</Alert>
	{/if}

	<section
		aria-labelledby="details-heading"
		class="print-hide mb-10 rounded-card border border-border bg-surface p-5 sm:p-6"
	>
		<div class="flex flex-wrap items-center justify-between gap-3">
			<h2 id="details-heading" class="font-display text-xl text-ink">Service details</h2>
			{#if canEdit && headerDirty}
				<span class="text-xs font-medium tracking-wide text-accent uppercase">Unsaved changes</span>
			{/if}
		</div>

		<form
			class="mt-5 grid gap-4 sm:grid-cols-2 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)_minmax(0,1fr)_10rem_auto] lg:items-end"
			onsubmit={saveHeader}
		>
			<TextField
				label="Title"
				name="setlist-title"
				required
				disabled={headerLocked}
				bind:value={headerDraft.title}
			/>
			<TextField
				label="Service date"
				name="setlist-date"
				type="date"
				disabled={headerLocked}
				bind:value={headerDraft.service_date}
			/>
			<TextField
				label="Service type"
				name="setlist-type"
				placeholder="Sunday morning"
				list="service-types"
				disabled={headerLocked}
				bind:value={headerDraft.service_type}
			/>
			<datalist id="service-types">
				<option value="Sunday morning"></option>
				<option value="Sunday evening"></option>
				<option value="Midweek"></option>
				<option value="Special event"></option>
			</datalist>
			<Select
				label="Status"
				name="setlist-status"
				disabled={!canEdit}
				bind:value={headerDraft.status}
			>
				<option value="draft">Draft</option>
				<option value="ready">Ready</option>
				<option value="archived">Archived</option>
			</Select>
			{#if canEdit}
				<Button type="submit" loading={savingHeader} disabled={!headerDirty}>Save</Button>
			{/if}
		</form>
	</section>

	<div class="grid gap-8 lg:grid-cols-[minmax(0,1fr)_19rem] lg:items-start">
		<section aria-labelledby="items-heading">
			<div class="mb-4 flex flex-wrap items-end justify-between gap-3">
				<div>
					<h2 id="items-heading" class="font-display text-xl text-ink">Run sheet</h2>
					<p class="mt-0.5 text-sm text-muted">
						{items.length}
						{items.length === 1 ? 'item' : 'items'}{#if canEditItems}<span aria-hidden="true"
								>&nbsp;·&nbsp;</span
							>Drag the handle or use the arrows to reorder.{/if}
					</p>
				</div>
				{#if reordering}
					<span role="status" class="flex items-center gap-2 text-sm text-muted">
						<span
							class="size-3.5 animate-spin rounded-full border-2 border-current border-t-transparent"
							aria-hidden="true"
						></span>
						Saving order…
					</span>
				{/if}
			</div>

			{#if items.length === 0}
				<div class="rounded-card border border-dashed border-border px-6 py-12 text-center">
					<h3 class="font-display text-lg text-ink">Nothing planned yet</h3>
					<p class="mt-1 text-sm text-muted">
						{#if canEditItems}
							Add a song or something spoken below.
						{:else}
							This setlist has no items.
						{/if}
					</p>
				</div>
			{:else}
				<ol class="flex flex-col gap-3" aria-label="Setlist items">
					{#each items as item, index (item.id)}
						{@const chart = itemChart(item)}
						{@const itemHintList = hints.get(item.id) ?? []}
						<li
							class="rounded-card border bg-surface transition-colors {dragOverId === item.id
								? 'border-accent ring-2 ring-accent/40'
								: 'border-border'} {draggedId === item.id ? 'opacity-50' : ''}"
							ondragover={(event) => dragOver(event, item)}
							ondrop={(event) => drop(event, item)}
						>
							<div class="flex items-start gap-2 p-3.5 sm:gap-3 sm:p-4">
								<div class="flex shrink-0 items-center gap-1 pt-0.5">
									{#if canEditItems}
										<button
											type="button"
											tabindex="-1"
											aria-hidden="true"
											draggable="true"
											title="Drag to reorder"
											class="grid size-8 cursor-grab place-items-center rounded-md text-muted transition-colors hover:bg-surface-2 hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent active:cursor-grabbing"
											ondragstart={(event) => dragStart(event, item)}
											ondragend={dragEnd}
										>
											<svg
												viewBox="0 0 16 16"
												class="size-4"
												fill="currentColor"
												aria-hidden="true"
											>
												<circle cx="6" cy="4" r="1.2" />
												<circle cx="10" cy="4" r="1.2" />
												<circle cx="6" cy="8" r="1.2" />
												<circle cx="10" cy="8" r="1.2" />
												<circle cx="6" cy="12" r="1.2" />
												<circle cx="10" cy="12" r="1.2" />
											</svg>
										</button>
									{/if}
									<span class="w-7 text-right font-mono text-xs text-muted" aria-hidden="true">
										{String(index + 1).padStart(2, '0')}
									</span>
								</div>

								<div class="min-w-0 flex-1">
									<div class="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
										<div class="min-w-0">
											<p class="font-medium text-ink">
												{itemTitle(item)}
												{#if item.kind === 'note'}
													<span
														class="ml-1.5 rounded bg-surface-2 px-1.5 py-0.5 align-middle text-[0.65rem] font-semibold tracking-wide text-muted uppercase"
														>Other</span
													>
												{:else if item.song?.artist}
													<span class="ml-1.5 text-sm font-normal text-muted"
														>· {item.song.artist}</span
													>
												{/if}
											</p>
											<div
												class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted"
											>
												{#if item.kind === 'song'}
													{#if itemKey(item)}
														<span>
															Key <span class="font-mono font-semibold text-accent"
																>{itemKey(item)}</span
															>
														</span>
													{/if}
													{#if item.capo_override}
														<span>Capo {item.capo_override}</span>
													{/if}
													{#if itemTempo(item)}
														<span>{itemTempo(item)} bpm</span>
													{/if}
												{/if}
												<span>{formatDuration(itemDuration(item))}</span>
											</div>
											{#if item.notes}
												<p class="mt-2 text-sm whitespace-pre-wrap text-muted">{item.notes}</p>
											{/if}
											{#if itemHintList.length > 0}
												<ul class="mt-2 flex flex-col gap-1">
													{#each itemHintList as hint (hint)}
														<li class="flex items-start gap-1.5 text-xs text-muted">
															<svg
																viewBox="0 0 16 16"
																class="mt-0.5 size-3.5 shrink-0 text-accent"
																fill="none"
																aria-hidden="true"
															>
																<path
																	d="M8 2.5 9.4 6l3.6 1.4L9.4 8.8 8 12.3 6.6 8.8 3 7.4 6.6 6 8 2.5Z"
																	stroke="currentColor"
																	stroke-width="1.2"
																	stroke-linejoin="round"
																/>
															</svg>
															<span>{hint}</span>
														</li>
													{/each}
												</ul>
											{/if}
										</div>

										<div class="flex shrink-0 flex-wrap items-center gap-1">
											{#if canEditItems}
												<button
													type="button"
													class="grid size-9 place-items-center rounded-md text-muted ring-1 ring-border transition-colors hover:bg-surface-2 hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40"
													aria-label="Move {itemTitle(item)} up"
													disabled={index === 0 || reordering}
													onclick={() => move(index, -1)}
												>
													<svg viewBox="0 0 20 20" fill="none" class="size-4" aria-hidden="true">
														<path
															d="m5 12 5-5 5 5"
															stroke="currentColor"
															stroke-width="1.7"
															stroke-linecap="round"
															stroke-linejoin="round"
														/>
													</svg>
												</button>
												<button
													type="button"
													class="grid size-9 place-items-center rounded-md text-muted ring-1 ring-border transition-colors hover:bg-surface-2 hover:text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:cursor-not-allowed disabled:opacity-40"
													aria-label="Move {itemTitle(item)} down"
													disabled={index === items.length - 1 || reordering}
													onclick={() => move(index, 1)}
												>
													<svg viewBox="0 0 20 20" fill="none" class="size-4" aria-hidden="true">
														<path
															d="m5 8 5 5 5-5"
															stroke="currentColor"
															stroke-width="1.7"
															stroke-linecap="round"
															stroke-linejoin="round"
														/>
													</svg>
												</button>
											{/if}

											{#if item.kind === 'song'}
												<Button
													variant="ghost"
													aria-expanded={!!expanded[item.id]}
													aria-controls="chart-{item.id}"
													onclick={() => toggleExpanded(item)}
												>
													{expanded[item.id] ? 'Hide chart' : 'Chart'}
												</Button>
											{/if}

											{#if canEditItems}
												<Button variant="ghost" onclick={() => startEdit(item)}>Edit</Button>
												{#if confirmItemDelete === item.id}
													<Button
														variant="danger"
														loading={deletingItemId === item.id}
														onclick={() => removeItem(item)}>Confirm</Button
													>
													<Button variant="ghost" onclick={() => (confirmItemDelete = null)}
														>No</Button
													>
												{:else}
													<Button
														variant="ghost"
														class="text-danger-text hover:bg-danger-tint"
														onclick={() => (confirmItemDelete = item.id)}>Delete</Button
													>
												{/if}
											{/if}
										</div>
									</div>

									{#if expanded[item.id]}
										<div id="chart-{item.id}" class="mt-4 border-t border-border pt-4">
											{#if editingItem === item.id && canEditItems}
												<div class="mb-5 rounded-field bg-canvas/50 p-4">
													<h3 class="mb-3 text-sm font-semibold text-ink">Item settings</h3>
													<div class="grid gap-4 sm:grid-cols-2">
														{#if item.kind === 'note'}
															<TextField
																label="Title"
																name="item-title-{item.id}"
																bind:value={itemDraft.title}
															/>
														{:else}
															<TextField
																label="Key override"
																name="item-key-{item.id}"
																placeholder={item.song?.default_key ?? 'G'}
																hint="Leave blank to use the library key."
																bind:value={itemDraft.key}
															/>
															<Select
																label="Capo"
																name="item-capo-{item.id}"
																bind:value={itemDraft.capo}
															>
																<option value="0">No capo</option>
																{#each Array.from({ length: 11 }, (_, i) => i + 1) as fret (fret)}
																	<option value={String(fret)}>Fret {fret}</option>
																{/each}
															</Select>
														{/if}
														<TextField
															label="Duration"
															name="item-duration-{item.id}"
															type="number"
															min="0"
															step="5"
															inputmode="numeric"
															placeholder={item.kind === 'song' ? '240' : '60'}
															hint="Seconds. Leave blank for the default."
															bind:value={itemDraft.duration}
														/>
													</div>
													<Textarea
														label="Notes"
														name="item-notes-{item.id}"
														rows={2}
														class="mt-4"
														hint="Only on this setlist — the song library is untouched."
														bind:value={itemDraft.notes}
													/>

													{#if itemError}
														<div
															bind:this={itemErrorBox}
															tabindex="-1"
															class="mt-3 rounded-field outline-none"
														>
															<Alert>{itemError}</Alert>
														</div>
													{/if}

													<div class="mt-4 flex flex-wrap items-center gap-2">
														<Button loading={savingItem === item.id} onclick={() => saveItem(item)}
															>Save item</Button
														>
														<Button variant="ghost" onclick={() => (editingItem = null)}
															>Cancel</Button
														>
														{#if item.kind === 'song'}
															<Button
																variant="ghost"
																loading={savingItem === item.id}
																onclick={() => resetItem(item)}
															>
																Reset to library
															</Button>
														{/if}
													</div>
												</div>
											{/if}

											{#if item.kind === 'song' && chart}
												<div
													class="mb-4 flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted"
												>
													<span>
														Sounds in
														<span class="font-mono font-semibold text-accent"
															>{chart.soundingKey || '—'}</span
														>
													</span>
													{#if chart.capoFret > 0}
														<span>Capo {chart.capoFret} · play in</span>
														<span class="font-mono font-semibold text-accent"
															>{chart.shapeKey || '—'}</span
														>
													{/if}
												</div>
												<div class="print-chart overflow-x-auto">
													<ChartView chart={chart.chart} />
												</div>
											{:else if item.kind === 'song'}
												<p class="text-sm text-muted">
													The song behind this item is no longer available.
												</p>
											{/if}
										</div>
									{/if}
								</div>
							</div>
						</li>
					{/each}
				</ol>
			{/if}

			{#if canEditItems}
				<div class="mt-8 flex flex-col gap-6">
					<SongPicker workspaceId={session.activeWorkspaceId ?? ''} {addSong} />

					<section
						aria-labelledby="add-note-heading"
						class="rounded-card border border-border bg-surface p-5"
					>
						<h2 id="add-note-heading" class="font-display text-lg text-ink">Add another item</h2>
						<p class="mt-0.5 text-sm text-muted">
							Prayers, scripture readings, offerings, and transitions.
						</p>
						<form class="mt-4 flex flex-col gap-4" onsubmit={addNote}>
							<div class="grid gap-4 sm:grid-cols-2">
								<Select label="Type" name="note-type" value={noteType} onchange={chooseNoteType}>
									{#each notePresets as preset (preset)}
										<option value={preset}>{preset}</option>
									{/each}
									<option value="Other">Other</option>
								</Select>
								<TextField
									label="Title"
									name="note-title"
									placeholder={noteType === 'Other' ? 'Announcements' : noteType}
									bind:value={noteTitle}
								/>
							</div>
							<Textarea
								label="Notes"
								name="note-notes"
								rows={2}
								hint="Optional"
								bind:value={noteNotes}
							/>
							{#if noteError}
								<div bind:this={noteErrorBox} tabindex="-1" class="rounded-field outline-none">
									<Alert>{noteError}</Alert>
								</div>
							{/if}
							<div>
								<Button type="submit" variant="secondary" loading={addingNote}>Add item</Button>
							</div>
						</form>
					</section>
				</div>
			{/if}
		</section>

		<aside class="flex flex-col gap-6 lg:sticky lg:top-24">
			<section
				aria-labelledby="share-heading"
				class="print-hide rounded-card border border-border bg-surface p-5"
			>
				<h2 id="share-heading" class="font-display text-lg text-ink">Share</h2>

				{#if setlist.share_token}
					<p class="mt-1 text-sm text-muted">
						Anyone with this link can read the setlist, without signing in.
					</p>
					<a
						href={resolve('/s/[token]', { token: setlist.share_token })}
						target="_blank"
						rel="noopener noreferrer"
						class="mt-3 block truncate rounded-field bg-canvas px-3 py-2 font-mono text-xs text-accent ring-1 ring-border hover:underline"
					>
						{sharePath(setlist.share_token)}
					</a>
					<div class="mt-3 flex flex-wrap items-center gap-2">
						<Button variant="secondary" onclick={() => copyShareLink(setlist?.share_token ?? '')}>
							{copied ? 'Copied' : 'Copy link'}
						</Button>
						{#if canEdit}
							{#if confirmRevoke}
								<Button variant="danger" loading={revoking} onclick={revokeShare}>
									Yes, revoke
								</Button>
								<Button variant="ghost" onclick={() => (confirmRevoke = false)}>Cancel</Button>
							{:else}
								<Button
									variant="ghost"
									class="text-danger-text hover:bg-danger-tint"
									onclick={() => (confirmRevoke = true)}>Revoke</Button
								>
							{/if}
						{/if}
					</div>
				{:else if canEditItems}
					<p class="mt-1 text-sm text-muted">
						Create a read-only link for the band, a projector, or a guest. You can revoke it at any
						time.
					</p>
					<Button class="mt-4" variant="secondary" loading={sharing} onclick={createShare}>
						Create share link
					</Button>
				{:else if setlist.status === 'archived'}
					<p class="mt-1 text-sm text-muted">
						Archived setlists cannot be shared. Set the status to Draft or Ready first.
					</p>
				{:else}
					<p class="mt-1 text-sm text-muted">Only leaders can create a share link.</p>
				{/if}

				{#if shareError || copyError}
					<div bind:this={shareErrorBox} tabindex="-1" class="mt-3 rounded-field outline-none">
						{#if shareError}
							<Alert>{shareError}</Alert>
						{/if}
						{#if copyError}
							<div class={shareError ? 'mt-2' : ''}><Alert>{copyError}</Alert></div>
						{/if}
					</div>
				{/if}
			</section>

			<section
				aria-labelledby="at-a-glance-heading"
				class="rounded-card border border-border bg-surface p-5"
			>
				<h2 id="at-a-glance-heading" class="font-display text-lg text-ink">At a glance</h2>
				<dl class="mt-3 flex flex-col gap-2.5 text-sm">
					<div class="flex items-baseline justify-between gap-3">
						<dt class="text-muted">Songs</dt>
						<dd class="font-medium text-ink">{songCount}</dd>
					</div>
					<div class="flex items-baseline justify-between gap-3">
						<dt class="text-muted">Other items</dt>
						<dd class="font-medium text-ink">{noteCount}</dd>
					</div>
					<div class="flex items-baseline justify-between gap-3 border-t border-border pt-2.5">
						<dt class="text-muted">Estimated length</dt>
						<dd class="font-mono font-semibold text-accent">{formatDuration(total)}</dd>
					</div>
				</dl>
				<p class="mt-3 text-xs leading-relaxed text-muted">
					Songs count as 4 minutes and other items as 1 minute unless you set a duration on the
					item.
				</p>
			</section>

			<section aria-labelledby="keys-heading" class="print-hide text-sm text-muted">
				<h2 id="keys-heading" class="mb-2 text-xs font-semibold tracking-wide uppercase">
					Transitions
				</h2>
				<p class="leading-relaxed">
					Key and tempo hints appear between consecutive songs. They are a nudge, not a rule.
				</p>
			</section>
		</aside>
	</div>
{/if}
