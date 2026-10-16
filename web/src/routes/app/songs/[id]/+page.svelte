<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		ApiError,
		createArrangement,
		createLink,
		deleteArrangement,
		deleteAsset,
		deleteSong,
		downloadAsset,
		duplicateSong,
		exportSong,
		getSong,
		listArrangements,
		listAssets,
		updateArrangement,
		updateSong,
		uploadPdf
	} from '#lib/api';
	import type { Arrangement, Asset, Song, UpdateSongInput } from '#lib/api';
	import { capo, detectKey, parseChordPro, transpose } from '#lib/chord';
	import type { Accidental } from '#lib/chord';
	import { session } from '#lib/session.svelte';
	import { canWrite } from '#lib/roles';
	import { formatBytes, saveResponse } from '#lib/download';
	import Alert from '#lib/components/Alert.svelte';
	import ArrangementForm from '#lib/components/ArrangementForm.svelte';
	import Button from '#lib/components/Button.svelte';
	import ChartView from '#lib/components/ChartView.svelte';
	import Select from '#lib/components/Select.svelte';
	import TextField from '#lib/components/TextField.svelte';
	import Textarea from '#lib/components/Textarea.svelte';
	import type { PageProps } from './$types';

	const MAX_UPLOAD_BYTES = 10 * 1024 * 1024;

	let { params }: PageProps = $props();

	const songId = $derived(params.id);

	let song = $state<Song | null>(null);
	let arrangements = $state<Arrangement[]>([]);
	let assets = $state<Asset[]>([]);
	const links = $derived(assets.filter((asset) => asset.kind === 'link'));
	const pdfs = $derived(assets.filter((asset) => asset.kind === 'pdf'));

	let loading = $state(true);
	let error = $state('');
	let errorBox = $state<HTMLElement>();

	// View-time transforms; the stored chordpro is never mutated.
	let semitones = $state(0);
	let accidentalPref = $state<'auto' | Accidental>('auto');
	let capoValue = $state('0');
	const capoFret = $derived(Number(capoValue) || 0);

	let editing = $state(false);
	let draft = $state({
		title: '',
		artist: '',
		default_key: '',
		tempo: '',
		time_signature: '',
		tags: '',
		ccli_number: '',
		copyright: '',
		chordpro: ''
	});
	let savingEdit = $state(false);

	let confirmId = $state<string | null>(null);
	let busyId = $state<string | null>(null);

	let addingArrangement = $state(false);
	let editingArrangementId = $state<string | null>(null);
	let arrangementBusy = $state(false);

	let linkUrl = $state('');
	let linkLabel = $state('');
	let linkError = $state('');
	let addingLink = $state(false);

	let pdfError = $state('');
	let uploadingPdf = $state(false);

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);
	const canEdit = $derived(canWrite(activeWorkspace?.role));

	const preferArg = $derived(accidentalPref === 'auto' ? undefined : accidentalPref);
	const baseChart = $derived(song ? parseChordPro(song.chordpro) : null);
	const transposed = $derived(baseChart ? transpose(baseChart, semitones, preferArg) : null);
	const capoResult = $derived(transposed ? capo(transposed, capoFret, preferArg) : null);
	const displayChart = $derived(capoFret > 0 ? (capoResult?.shapes ?? null) : transposed);
	const soundingKey = $derived(
		capoFret > 0 ? (capoResult?.soundingKey ?? '') : transposed ? (detectKey(transposed) ?? '') : ''
	);
	const shapeKey = $derived(
		capoFret > 0 ? (capoResult?.shapeKey ?? '') : transposed ? (detectKey(transposed) ?? '') : ''
	);
	const draftChart = $derived(parseChordPro(draft.chordpro));

	async function load() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) {
			loading = false;
			return;
		}
		loading = true;
		error = '';
		try {
			song = await getSong(workspaceId, songId);
			arrangements = await listArrangements(workspaceId, songId);
			// Assets are supplementary: a failure here (e.g. the endpoint is not
			// deployed yet) must not blank the song or block the page.
			try {
				assets = await listAssets(workspaceId, songId);
			} catch {
				assets = [];
			}
		} catch (cause) {
			song = null;
			arrangements = [];
			assets = [];
			error =
				cause instanceof ApiError
					? cause.status === 404
						? 'This song was not found in the current workspace.'
						: cause.message
					: 'Could not load the song.';
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		void songId;
		void session.activeWorkspaceId;
		void load();
	});

	$effect(() => {
		if (error) errorBox?.focus();
	});

	function step(delta: number) {
		semitones = Math.max(-11, Math.min(11, semitones + delta));
	}

	function resetTranspose() {
		semitones = 0;
		capoValue = '0';
	}

	function startEdit() {
		if (!song) return;
		draft = {
			title: song.title,
			artist: song.artist ?? '',
			default_key: song.default_key ?? '',
			tempo: song.tempo === null ? '' : String(song.tempo),
			time_signature: song.time_signature,
			tags: song.tags.join(', '),
			ccli_number: song.ccli_number ?? '',
			copyright: song.copyright ?? '',
			chordpro: song.chordpro
		};
		editing = true;
	}

	async function saveEdit() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song) return;
		if (!draft.title.trim()) {
			error = 'Title must not be empty.';
			return;
		}
		const parsedTempo = Number.parseInt(draft.tempo, 10);
		const payload: UpdateSongInput = {
			title: draft.title.trim(),
			artist: draft.artist.trim() || null,
			default_key: draft.default_key.trim() || null,
			tempo: Number.isFinite(parsedTempo) ? parsedTempo : null,
			time_signature: draft.time_signature.trim() || null,
			tags: draft.tags
				.split(',')
				.map((tag) => tag.trim())
				.filter(Boolean),
			ccli_number: draft.ccli_number.trim() || null,
			copyright: draft.copyright.trim() || null,
			chordpro: draft.chordpro
		};
		savingEdit = true;
		error = '';
		try {
			song = await updateSong(workspaceId, song.id, payload);
			editing = false;
			semitones = 0;
			capoValue = '0';
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not save the changes.';
		} finally {
			savingEdit = false;
		}
	}

	async function removeSong() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song) return;
		busyId = 'song';
		try {
			await deleteSong(workspaceId, song.id);
			await goto(resolve('/app/songs'));
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not delete the song.';
			confirmId = null;
		} finally {
			busyId = null;
		}
	}

	async function duplicate() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song) return;
		busyId = 'duplicate';
		try {
			const copy = await duplicateSong(workspaceId, song.id);
			await goto(resolve('/app/songs/[id]', { id: copy.id }));
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not duplicate the song.';
		} finally {
			busyId = null;
		}
	}

	async function exportAs(format: 'chordpro' | 'text') {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song) return;
		busyId = `export-${format}`;
		try {
			const response = await exportSong(workspaceId, song.id, format);
			await saveResponse(response, `${song.title || 'song'}.${format === 'text' ? 'txt' : 'cho'}`);
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not export the song.';
		} finally {
			busyId = null;
		}
	}

	async function addArrangement(value: { name: string; key: string; chordpro: string }) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song) return;
		arrangementBusy = true;
		try {
			const created = await createArrangement(workspaceId, song.id, {
				name: value.name,
				key: value.key || null,
				chordpro: value.chordpro || song.chordpro
			});
			arrangements = [...arrangements, created].sort((a, b) => a.name.localeCompare(b.name));
			addingArrangement = false;
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not add the arrangement.';
		} finally {
			arrangementBusy = false;
		}
	}

	async function saveArrangement(
		arrangement: Arrangement,
		value: { name: string; key: string; chordpro: string }
	) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		arrangementBusy = true;
		try {
			const updated = await updateArrangement(workspaceId, arrangement.id, {
				name: value.name,
				key: value.key || null,
				chordpro: value.chordpro
			});
			arrangements = arrangements.map((item) => (item.id === updated.id ? updated : item));
			editingArrangementId = null;
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not update the arrangement.';
		} finally {
			arrangementBusy = false;
		}
	}

	async function removeArrangement(id: string) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		busyId = id;
		try {
			await deleteArrangement(workspaceId, id);
			arrangements = arrangements.filter((item) => item.id !== id);
			confirmId = null;
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not delete the arrangement.';
		} finally {
			busyId = null;
		}
	}

	async function addLink() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song) return;
		const url = linkUrl.trim();
		if (!/^https?:\/\//i.test(url)) {
			linkError = 'Enter a link starting with http:// or https://.';
			return;
		}
		linkError = '';
		addingLink = true;
		try {
			const asset = await createLink(workspaceId, song.id, url, linkLabel.trim() || undefined);
			assets = [...assets, asset];
			linkUrl = '';
			linkLabel = '';
		} catch (cause) {
			linkError = cause instanceof ApiError ? cause.message : 'Could not add the link.';
		} finally {
			addingLink = false;
		}
	}

	async function removeAsset(id: string) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		busyId = id;
		try {
			await deleteAsset(workspaceId, id);
			assets = assets.filter((asset) => asset.id !== id);
			confirmId = null;
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not remove the item.';
		} finally {
			busyId = null;
		}
	}

	async function handleFile(files: FileList | null) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !song || !files || files.length === 0) return;
		const file = files[0];
		pdfError = '';
		if (file.type !== 'application/pdf') {
			pdfError = 'Only PDF files are accepted.';
			return;
		}
		if (file.size > MAX_UPLOAD_BYTES) {
			pdfError = `That file is ${formatBytes(file.size)}. The limit is 10 MB.`;
			return;
		}
		uploadingPdf = true;
		try {
			const asset = await uploadPdf(workspaceId, song.id, file);
			assets = [...assets, asset];
		} catch (cause) {
			if (cause instanceof ApiError && cause.status === 413) {
				pdfError = 'That file is too large. The limit is 10 MB.';
			} else if (cause instanceof ApiError && cause.status === 415) {
				pdfError = 'The server only accepts valid PDF files.';
			} else {
				pdfError = cause instanceof ApiError ? cause.message : 'Could not upload the file.';
			}
		} finally {
			uploadingPdf = false;
		}
	}

	async function downloadPdf(asset: Asset) {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		busyId = asset.id;
		try {
			const response = await downloadAsset(workspaceId, asset.id);
			await saveResponse(response, asset.filename ?? 'file.pdf');
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not download the file.';
		} finally {
			busyId = null;
		}
	}
</script>

{#if loading}
	<div role="status" class="flex items-center gap-3 py-10 text-muted">
		<span
			class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
			aria-hidden="true"
		></span>
		Loading song…
	</div>
{:else if !song}
	<div class="max-w-lg">
		<h1 class="font-display text-display text-ink">Song not found</h1>
		<p class="mt-2 text-muted">{error || 'This song could not be loaded.'}</p>
		<a
			href={resolve('/app/songs')}
			class="mt-6 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
		>
			Back to songs
		</a>
	</div>
{:else}
	<nav aria-label="Breadcrumb" class="mb-6 text-sm text-muted">
		<a href={resolve('/app/songs')} class="font-medium text-accent hover:underline">Songs</a>
		<span class="mx-2" aria-hidden="true">/</span>
		<span class="text-ink">{song.title}</span>
	</nav>

	{#if error}
		<div bind:this={errorBox} tabindex="-1" class="mb-6 rounded-field outline-none">
			<Alert>{error}</Alert>
		</div>
	{/if}
	{#if !canEdit}
		<Alert variant="info" class="mb-6">
			Your role in this workspace is read-only. You can view and export this song, but not change
			it.
		</Alert>
	{/if}

	<header class="mb-8 flex flex-wrap items-start justify-between gap-4">
		<div class="min-w-0">
			<h1 class="font-display text-display text-ink">{song.title}</h1>
			<p class="mt-1 text-muted">
				{song.artist ?? 'Unknown artist'}
			</p>
			<div class="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1.5 text-sm text-muted">
				{#if song.default_key}
					<span
						>Key <span class="font-mono font-semibold text-accent">{song.default_key}</span></span
					>
				{/if}
				{#if song.tempo}
					<span>{song.tempo} bpm</span>
				{/if}
				{#if song.time_signature}
					<span>{song.time_signature}</span>
				{/if}
				{#if song.ccli_number}
					<span>CCLI {song.ccli_number}</span>
				{/if}
				{#if song.copyright}
					<span>{song.copyright}</span>
				{/if}
			</div>
			{#if song.tags.length > 0}
				<div class="mt-3 flex flex-wrap gap-1.5">
					{#each song.tags as tag (tag)}
						<span class="rounded-full bg-surface-2 px-2.5 py-0.5 text-xs text-muted">{tag}</span>
					{/each}
				</div>
			{/if}
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<Button
				variant="secondary"
				loading={busyId === 'export-chordpro'}
				onclick={() => exportAs('chordpro')}
			>
				Export ChordPro
			</Button>
			<Button
				variant="secondary"
				loading={busyId === 'export-text'}
				onclick={() => exportAs('text')}
			>
				Export text
			</Button>
			{#if canEdit}
				{#if editing}
					<Button variant="ghost" onclick={() => (editing = false)}>Cancel</Button>
					<Button loading={savingEdit} onclick={saveEdit}>Save changes</Button>
				{:else}
					<Button variant="secondary" onclick={startEdit}>Edit</Button>
					<Button variant="secondary" loading={busyId === 'duplicate'} onclick={duplicate}>
						Duplicate
					</Button>
				{/if}
			{/if}
		</div>
	</header>

	<!-- Chord editor with live preview -->
	{#if editing}
		<section aria-labelledby="editor-heading" class="mb-10">
			<h2 id="editor-heading" class="mb-3 text-sm font-semibold tracking-wide text-muted uppercase">
				Edit song
			</h2>
			<div class="grid gap-6 lg:grid-cols-2 lg:items-start">
				<div class="flex flex-col gap-4">
					<div class="grid gap-4 sm:grid-cols-2">
						<TextField label="Title" name="edit-title" required bind:value={draft.title} />
						<TextField label="Artist" name="edit-artist" bind:value={draft.artist} />
						<TextField label="Key" name="edit-key" bind:value={draft.default_key} />
						<TextField
							label="Tempo"
							name="edit-tempo"
							type="number"
							inputmode="numeric"
							bind:value={draft.tempo}
						/>
						<TextField label="Time signature" name="edit-time" bind:value={draft.time_signature} />
						<TextField
							label="Tags"
							name="edit-tags"
							hint="Separate with commas."
							bind:value={draft.tags}
						/>
						<TextField label="CCLI number" name="edit-ccli" bind:value={draft.ccli_number} />
						<TextField label="Copyright" name="edit-copyright" bind:value={draft.copyright} />
					</div>
					<Textarea
						label="ChordPro source"
						name="edit-chordpro"
						rows={14}
						hint="Edits here change the saved chart. Transpose and capo stay view-only."
						bind:value={draft.chordpro}
					/>
				</div>
				<div class="rounded-card border border-border bg-surface p-5 lg:sticky lg:top-24">
					<h3 class="mb-4 text-sm font-semibold tracking-wide text-muted uppercase">Preview</h3>
					<div class="max-h-[70vh] overflow-auto">
						<ChartView chart={draftChart} />
					</div>
				</div>
			</div>
		</section>
	{/if}

	<div class="grid gap-10 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-start">
		<!-- Chart: transpose + capo -->
		<section aria-labelledby="chart-heading">
			<div class="mb-4 flex flex-wrap items-center justify-between gap-3">
				<h2 id="chart-heading" class="text-sm font-semibold tracking-wide text-muted uppercase">
					Chart
				</h2>
				{#if soundingKey || shapeKey}
					<p class="text-sm text-muted">
						{#if capoFret > 0}
							Capo {capoFret} · play in
							<span class="font-mono font-semibold text-accent">{shapeKey || '—'}</span> · sounds in
							<span class="font-mono font-semibold text-accent">{soundingKey || '—'}</span>
						{:else}
							Key <span class="font-mono font-semibold text-accent">{soundingKey || '—'}</span>
						{/if}
					</p>
				{/if}
			</div>

			<div
				class="mb-6 flex flex-wrap items-end gap-x-8 gap-y-4 rounded-card border border-border bg-surface p-4"
			>
				<div>
					<p id="transpose-label" class="mb-1.5 text-sm font-medium text-ink">Transpose</p>
					<div class="flex items-center gap-2" role="group" aria-labelledby="transpose-label">
						<Button
							variant="secondary"
							aria-label="Down one semitone"
							disabled={semitones <= -11}
							onclick={() => step(-1)}
						>
							−
						</Button>
						<span class="w-12 text-center font-mono text-sm text-ink">
							{semitones > 0 ? `+${semitones}` : semitones}
						</span>
						<Button
							variant="secondary"
							aria-label="Up one semitone"
							disabled={semitones >= 11}
							onclick={() => step(1)}
						>
							+
						</Button>
					</div>
				</div>

				<fieldset>
					<legend class="mb-1.5 text-sm font-medium text-ink">Note spelling</legend>
					<div
						class="inline-flex rounded-field bg-surface-2 p-1 ring-1 ring-border"
						role="group"
						aria-label="Note spelling"
					>
						{#each [['auto', 'Auto'], ['sharp', 'Sharps'], ['flat', 'Flats']] as [value, label] (value)}
							<button
								type="button"
								aria-pressed={accidentalPref === value}
								onclick={() => (accidentalPref = value as 'auto' | Accidental)}
								class="rounded-[0.45rem] px-3 py-1 text-sm font-medium transition-colors {accidentalPref ===
								value
									? 'bg-accent text-accent-ink'
									: 'text-muted hover:text-ink'}"
							>
								{label}
							</button>
						{/each}
					</div>
				</fieldset>

				<Select label="Capo" name="capo" bind:value={capoValue} class="w-40">
					<option value="0">No capo</option>
					{#each Array.from({ length: 11 }, (_, i) => i + 1) as fret (fret)}
						<option value={String(fret)}>Fret {fret}</option>
					{/each}
				</Select>

				{#if semitones !== 0 || capoFret !== 0}
					<Button variant="ghost" onclick={resetTranspose}>Reset</Button>
				{/if}
			</div>

			<div class="rounded-card border border-border bg-surface p-5 sm:p-6">
				{#if displayChart && displayChart.sections.some((section) => section.lines.length > 0)}
					<div class="overflow-x-auto">
						<ChartView chart={displayChart} />
					</div>
				{:else}
					<p class="py-8 text-center text-muted">
						This song has no chart yet.
						{#if canEdit}
							Use Edit to add ChordPro.
						{/if}
					</p>
				{/if}
			</div>
		</section>

		<aside class="flex flex-col gap-8">
			<!-- Arrangements -->
			<section aria-labelledby="arrangements-heading">
				<div class="mb-3 flex items-center justify-between gap-3">
					<h2
						id="arrangements-heading"
						class="text-sm font-semibold tracking-wide text-muted uppercase"
					>
						Arrangements
					</h2>
					{#if canEdit && !addingArrangement}
						<Button variant="ghost" onclick={() => (addingArrangement = true)}>Add</Button>
					{/if}
				</div>

				{#if arrangements.length === 0 && !addingArrangement}
					<p class="text-sm text-muted">
						No alternate arrangements yet. The main chart is the default.
					</p>
				{/if}

				<ul class="flex flex-col gap-3">
					{#each arrangements as arrangement (arrangement.id)}
						<li class="rounded-card border border-border bg-surface p-4">
							{#if editingArrangementId === arrangement.id}
								<ArrangementForm
									initialName={arrangement.name}
									initialKey={arrangement.key ?? ''}
									initialChordpro={arrangement.chordpro}
									submitLabel="Save"
									saving={arrangementBusy}
									onSubmit={(value) => saveArrangement(arrangement, value)}
									onCancel={() => (editingArrangementId = null)}
								/>
							{:else}
								<div class="flex items-center justify-between gap-3">
									<div class="min-w-0">
										<p class="truncate font-medium text-ink">{arrangement.name}</p>
										{#if arrangement.key}
											<p class="text-sm text-muted">
												Key <span class="font-mono text-accent">{arrangement.key}</span>
											</p>
										{/if}
									</div>
									{#if canEdit}
										<div class="flex shrink-0 items-center gap-1">
											<Button
												variant="ghost"
												onclick={() => (editingArrangementId = arrangement.id)}>Edit</Button
											>
											{#if confirmId === arrangement.id}
												<Button
													variant="danger"
													loading={busyId === arrangement.id}
													onclick={() => removeArrangement(arrangement.id)}>Confirm</Button
												>
												<Button variant="ghost" onclick={() => (confirmId = null)}>No</Button>
											{:else}
												<Button
													variant="ghost"
													class="text-danger-text hover:bg-danger-tint"
													onclick={() => (confirmId = arrangement.id)}>Delete</Button
												>
											{/if}
										</div>
									{/if}
								</div>
							{/if}
						</li>
					{/each}
				</ul>

				{#if addingArrangement}
					<div class="mt-3 rounded-card border border-border bg-surface p-4">
						<ArrangementForm
							initialChordpro={song.chordpro}
							submitLabel="Add arrangement"
							saving={arrangementBusy}
							onSubmit={addArrangement}
							onCancel={() => (addingArrangement = false)}
						/>
					</div>
				{/if}
			</section>

			<!-- Reference links -->
			<section aria-labelledby="links-heading">
				<h2
					id="links-heading"
					class="mb-3 text-sm font-semibold tracking-wide text-muted uppercase"
				>
					Reference links
				</h2>

				{#if links.length === 0}
					<p class="text-sm text-muted">No links added.</p>
				{:else}
					<ul class="mb-3 flex flex-col gap-2">
						{#each links as link (link.id)}
							<li
								class="flex items-center justify-between gap-3 rounded-card border border-border bg-surface px-4 py-2.5"
							>
								<a
									href={link.url ?? '#'}
									target="_blank"
									rel="external noopener noreferrer"
									class="min-w-0 flex-1 truncate text-accent hover:underline"
								>
									{link.label || link.url}
								</a>
								{#if canEdit}
									{#if confirmId === link.id}
										<Button
											variant="danger"
											loading={busyId === link.id}
											onclick={() => removeAsset(link.id)}>Confirm</Button
										>
										<Button variant="ghost" onclick={() => (confirmId = null)}>No</Button>
									{:else}
										<Button
											variant="ghost"
											class="shrink-0 text-danger-text hover:bg-danger-tint"
											onclick={() => (confirmId = link.id)}>Delete</Button
										>
									{/if}
								{/if}
							</li>
						{/each}
					</ul>
				{/if}

				{#if canEdit}
					<form
						class="flex flex-col gap-3"
						onsubmit={(event) => {
							event.preventDefault();
							void addLink();
						}}
					>
						<TextField
							label="Link URL"
							name="link-url"
							type="url"
							placeholder="https://example.com/chords"
							bind:value={linkUrl}
						/>
						<TextField label="Label" name="link-label" hint="Optional" bind:value={linkLabel} />
						{#if linkError}
							<Alert>{linkError}</Alert>
						{/if}
						<Button type="submit" variant="secondary" loading={addingLink}>Add link</Button>
					</form>
				{/if}
			</section>

			<!-- PDFs -->
			<section aria-labelledby="pdf-heading">
				<h2 id="pdf-heading" class="mb-3 text-sm font-semibold tracking-wide text-muted uppercase">
					PDF files
				</h2>

				{#if pdfs.length === 0}
					<p class="text-sm text-muted">No PDFs uploaded.</p>
				{:else}
					<ul class="mb-3 flex flex-col gap-2">
						{#each pdfs as pdf (pdf.id)}
							<li
								class="flex items-center justify-between gap-3 rounded-card border border-border bg-surface px-4 py-2.5"
							>
								<span class="min-w-0">
									<span class="block truncate text-ink">{pdf.filename ?? 'file.pdf'}</span>
									<span class="text-xs text-muted">{formatBytes(pdf.size_bytes)}</span>
								</span>
								<div class="flex shrink-0 items-center gap-1">
									<Button
										variant="ghost"
										loading={busyId === pdf.id}
										onclick={() => downloadPdf(pdf)}>Download</Button
									>
									{#if canEdit}
										{#if confirmId === pdf.id}
											<Button
												variant="danger"
												loading={busyId === pdf.id}
												onclick={() => removeAsset(pdf.id)}>Confirm</Button
											>
											<Button variant="ghost" onclick={() => (confirmId = null)}>No</Button>
										{:else}
											<Button
												variant="ghost"
												class="text-danger-text hover:bg-danger-tint"
												onclick={() => (confirmId = pdf.id)}>Delete</Button
											>
										{/if}
									{/if}
								</div>
							</li>
						{/each}
					</ul>
				{/if}

				{#if canEdit}
					<label class="block">
						<span class="mb-1.5 block text-sm font-medium text-ink">Upload a PDF</span>
						<input
							type="file"
							accept="application/pdf,.pdf"
							aria-label="Upload a PDF"
							class="block w-full text-sm text-muted file:mr-3 file:min-h-9 file:rounded-field file:border-0 file:bg-surface-2 file:px-3 file:text-ink file:transition-colors hover:file:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
							onchange={(event) => {
								void handleFile(event.currentTarget.files);
								event.currentTarget.value = '';
							}}
						/>
						<span class="mt-1.5 block text-xs text-muted">PDF only, up to 10 MB.</span>
					</label>
					{#if uploadingPdf}
						<p role="status" class="mt-3 flex items-center gap-2 text-sm text-muted">
							<span
								class="size-3.5 animate-spin rounded-full border-2 border-current border-t-transparent"
								aria-hidden="true"
							></span>
							Uploading…
						</p>
					{/if}
					{#if pdfError}
						<div class="mt-3"><Alert>{pdfError}</Alert></div>
					{/if}
				{/if}
			</section>
		</aside>
	</div>

	{#if canEdit}
		<section aria-labelledby="danger-heading" class="mt-12 border-t border-border pt-6">
			<h2 id="danger-heading" class="sr-only">Delete song</h2>
			{#if confirmId === 'song'}
				<div class="flex flex-wrap items-center gap-3">
					<p class="text-sm text-muted">Delete this song for everyone in the workspace?</p>
					<Button variant="danger" loading={busyId === 'song'} onclick={removeSong}>
						Yes, delete
					</Button>
					<Button variant="ghost" onclick={() => (confirmId = null)}>Cancel</Button>
				</div>
			{:else}
				<Button variant="danger" onclick={() => (confirmId = 'song')}>Delete song</Button>
			{/if}
		</section>
	{/if}
{/if}
