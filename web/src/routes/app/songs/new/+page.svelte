<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, createSong } from '#lib/api';
	import { fromPlainText, parseChordPro } from '#lib/chord';
	import { session } from '#lib/session.svelte';
	import { canWrite } from '#lib/roles';
	import Alert from '#lib/components/Alert.svelte';
	import Button from '#lib/components/Button.svelte';
	import ChartView from '#lib/components/ChartView.svelte';
	import TextField from '#lib/components/TextField.svelte';
	import Textarea from '#lib/components/Textarea.svelte';

	type SourceMode = 'plain' | 'chordpro';

	let sourceMode = $state<SourceMode>('plain');
	let source = $state('');

	let title = $state('');
	let artist = $state('');
	let defaultKey = $state('');
	let tempo = $state('');
	let timeSignature = $state('4/4');
	let tags = $state('');
	let ccli = $state('');
	let copyright = $state('');

	let titleTouched = $state(false);
	let saving = $state(false);
	let error = $state('');
	let errorBox = $state<HTMLElement>();

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);
	const canEdit = $derived(canWrite(activeWorkspace?.role));

	// Plain text is converted up front, so the stored song is always ChordPro.
	const chordproSource = $derived(sourceMode === 'plain' ? fromPlainText(source) : source);
	const chart = $derived(parseChordPro(chordproSource));
	const hasContent = $derived(chart.sections.some((section) => section.lines.length > 0));

	$effect(() => {
		const metaTitle = chart.meta.title;
		if (!titleTouched && metaTitle) title = metaTitle;
	});

	$effect(() => {
		if (error) errorBox?.focus();
	});

	function parseTags(value: string): string[] {
		return value
			.split(',')
			.map((tag) => tag.trim())
			.filter(Boolean);
	}

	function parseTempo(value: string): number | null {
		const parsed = Number.parseInt(value, 10);
		return Number.isFinite(parsed) ? parsed : null;
	}

	async function save(event: SubmitEvent) {
		event.preventDefault();
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !canEdit) return;
		const finalTitle = title.trim() || chart.meta.title?.trim() || '';
		if (!finalTitle) {
			error = 'Add a title before saving.';
			return;
		}
		error = '';
		saving = true;
		try {
			const song = await createSong(workspaceId, {
				title: finalTitle,
				artist: artist.trim() || null,
				default_key: defaultKey.trim() || null,
				tempo: parseTempo(tempo),
				time_signature: timeSignature.trim() || null,
				tags: parseTags(tags),
				ccli_number: ccli.trim() || null,
				copyright: copyright.trim() || null,
				chordpro: chordproSource
			});
			await goto(resolve('/app/songs/[id]', { id: song.id }));
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Could not save the song.';
		} finally {
			saving = false;
		}
	}
</script>

<nav aria-label="Breadcrumb" class="mb-6 text-sm text-muted">
	<a href={resolve('/app/songs')} class="font-medium text-accent hover:underline">Songs</a>
	<span class="mx-2" aria-hidden="true">/</span>
	<span>New song</span>
</nav>

<header class="mb-8">
	<h1 class="font-display text-display text-ink">New song</h1>
	<p class="mt-2 max-w-prose text-muted">
		Paste a plain-text chart or ChordPro source, check the preview, then save it to the library.
	</p>
</header>

{#if !canEdit}
	<div class="max-w-lg">
		<Alert variant="info">Your role in this workspace is read-only, so you cannot add songs.</Alert>
		<a
			href={resolve('/app/songs')}
			class="mt-5 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
		>
			Back to songs
		</a>
	</div>
{:else}
	<form class="grid gap-8 lg:grid-cols-2 lg:items-start" onsubmit={save}>
		<div class="flex flex-col gap-6">
			<fieldset>
				<legend class="mb-2 text-sm font-medium text-ink">Source format</legend>
				<div class="inline-flex rounded-field bg-surface-2 p-1 ring-1 ring-border">
					<button
						type="button"
						aria-pressed={sourceMode === 'plain'}
						onclick={() => (sourceMode = 'plain')}
						class="rounded-[0.45rem] px-3.5 py-1.5 text-sm font-medium transition-colors {sourceMode ===
						'plain'
							? 'bg-accent text-accent-ink'
							: 'text-muted hover:text-ink'}"
					>
						Plain text
					</button>
					<button
						type="button"
						aria-pressed={sourceMode === 'chordpro'}
						onclick={() => (sourceMode = 'chordpro')}
						class="rounded-[0.45rem] px-3.5 py-1.5 text-sm font-medium transition-colors {sourceMode ===
						'chordpro'
							? 'bg-accent text-accent-ink'
							: 'text-muted hover:text-ink'}"
					>
						ChordPro
					</button>
				</div>
			</fieldset>

			<Textarea
				label={sourceMode === 'plain' ? 'Chart' : 'ChordPro source'}
				name="source"
				rows={16}
				hint={sourceMode === 'plain'
					? 'Chords on their own line above the lyric line. Blank lines separate sections.'
					: 'Uses [chord] markers and {directives}.'}
				placeholder={sourceMode === 'plain'
					? 'Verse 1:\nG                 C\nAmazing grace how sweet the sound'
					: '{title: Amazing Grace}\n{comment: Verse 1}\n[G]Amazing [C]grace'}
				bind:value={source}
			/>

			<div class="grid gap-4 sm:grid-cols-2">
				<TextField
					label="Title"
					name="title"
					required
					placeholder="Amazing Grace"
					bind:value={title}
					oninput={() => (titleTouched = true)}
				/>
				<TextField label="Artist" name="artist" placeholder="John Newton" bind:value={artist} />
				<TextField label="Key" name="default_key" placeholder="G" bind:value={defaultKey} />
				<TextField
					label="Tempo"
					name="tempo"
					type="number"
					inputmode="numeric"
					placeholder="72"
					bind:value={tempo}
				/>
				<TextField
					label="Time signature"
					name="time_signature"
					placeholder="4/4"
					bind:value={timeSignature}
				/>
				<TextField
					label="Tags"
					name="tags"
					placeholder="grace, classic"
					hint="Separate with commas."
					bind:value={tags}
				/>
				<TextField label="CCLI number" name="ccli_number" placeholder="1234567" bind:value={ccli} />
				<TextField
					label="Copyright"
					name="copyright"
					placeholder="© 1779 Public domain"
					bind:value={copyright}
				/>
			</div>

			{#if error}
				<div bind:this={errorBox} tabindex="-1" class="rounded-field outline-none">
					<Alert>{error}</Alert>
				</div>
			{/if}

			<div class="flex flex-wrap gap-3">
				<Button type="submit" size="lg" loading={saving}>
					{saving ? 'Saving…' : 'Save song'}
				</Button>
				<a
					href={resolve('/app/songs')}
					class="inline-flex min-h-13 items-center rounded-field px-6 font-medium text-muted transition-colors hover:bg-surface-2 hover:text-ink"
				>
					Cancel
				</a>
			</div>
		</div>

		<section
			aria-labelledby="preview-heading"
			class="rounded-card border border-border bg-surface p-5 lg:sticky lg:top-24"
		>
			<h2
				id="preview-heading"
				class="mb-4 text-sm font-semibold tracking-wide text-muted uppercase"
			>
				Preview
			</h2>
			<div class="max-h-[70vh] overflow-auto">
				{#if hasContent}
					<ChartView {chart} />
				{:else}
					<p class="py-8 text-center text-sm text-muted">
						Start typing a chart to see the preview here.
					</p>
				{/if}
			</div>
		</section>
	</form>
{/if}
