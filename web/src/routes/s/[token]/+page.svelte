<script lang="ts">
	import { resolve } from '$app/paths';
	import { ApiError, getSharedSetlist } from '#lib/api';
	import type { SetlistDetail } from '#lib/api';
	import {
		formatDuration,
		formatServiceDate,
		itemChart,
		itemDuration,
		itemKey,
		itemTempo,
		itemTitle,
		totalDuration
	} from '#lib/setlists';
	import Brand from '#lib/components/Brand.svelte';
	import Button from '#lib/components/Button.svelte';
	import ChartView from '#lib/components/ChartView.svelte';
	import type { PageProps } from './$types';

	let { params }: PageProps = $props();

	const token = $derived(params.token);
	let detail = $state<SetlistDetail | null>(null);
	let loading = $state(true);
	let error = $state('');
	let requestId = 0;

	async function load() {
		const current = ++requestId;
		loading = true;
		error = '';
		try {
			const data = await getSharedSetlist(token);
			if (current !== requestId) return;
			detail = data;
		} catch (cause) {
			if (current !== requestId) return;
			detail = null;
			error =
				cause instanceof ApiError && cause.status === 404
					? 'This share link is unknown or has been revoked.'
					: cause instanceof ApiError
						? cause.message
						: 'Could not load this setlist.';
		} finally {
			if (current === requestId) loading = false;
		}
	}

	$effect(() => {
		void token;
		void load();
	});
</script>

<svelte:head>
	<title>{detail ? `${detail.setlist.title} · OpenSetlist` : 'Setlist · OpenSetlist'}</title>
</svelte:head>

<div class="min-h-dvh">
	<header class="print-hide border-b border-border bg-canvas/85">
		<div class="mx-auto flex h-16 w-full max-w-5xl items-center justify-between gap-4 px-4 sm:px-6">
			<Brand />
			<span class="text-sm text-muted">Shared setlist</span>
		</div>
	</header>

	<main id="main" class="mx-auto w-full max-w-4xl px-4 py-8 sm:px-6 sm:py-12">
		{#if loading}
			<div role="status" class="flex items-center gap-3 py-12 text-muted">
				<span
					class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
					aria-hidden="true"
				></span>
				Loading setlist…
			</div>
		{:else if !detail}
			<div class="mx-auto max-w-md rounded-card border border-border bg-surface p-7 text-center">
				<span
					class="mx-auto grid size-12 place-items-center rounded-full bg-surface-2 text-muted"
					aria-hidden="true"
				>
					<svg viewBox="0 0 24 24" fill="none" class="size-6">
						<path
							d="M9 4H6a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-8"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
						/>
						<path
							d="m14 4 6 6M20 4l-6 6"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
						/>
					</svg>
				</span>
				<h1 class="mt-4 font-display text-2xl text-ink">Setlist unavailable</h1>
				<p class="mt-2 text-muted">{error}</p>
				<p class="mt-1 text-sm text-muted">
					If someone shared this setlist with you, ask them for a fresh link.
				</p>
				<a
					href={resolve('/')}
					class="mt-6 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
				>
					Go to OpenSetlist
				</a>
			</div>
		{:else}
			{@const setlist = detail.setlist}
			{@const items = detail.items}
			{@const total = totalDuration(items)}

			<div class="mb-6 flex flex-wrap items-start justify-between gap-4">
				<nav aria-label="Breadcrumb" class="print-hide text-sm text-muted">
					<a href={resolve('/')} class="font-medium text-accent hover:underline">OpenSetlist</a>
					<span class="mx-2" aria-hidden="true">/</span>
					<span>Shared setlist</span>
				</nav>
				<Button variant="secondary" class="print-hide" onclick={() => window.print()}>
					<svg viewBox="0 0 24 24" fill="none" class="size-4" aria-hidden="true">
						<path
							d="M7 9V4h10v5M7 17H5a1 1 0 0 1-1-1v-6a1 1 0 0 1 1-1h14a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-2"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							stroke-linejoin="round"
						/>
						<rect
							x="7"
							y="14"
							width="10"
							height="6"
							rx="1"
							stroke="currentColor"
							stroke-width="1.6"
						/>
					</svg>
					Print
				</Button>
			</div>

			<div class="print-setlist">
				<header class="mb-8 border-b border-border pb-6">
					<p class="text-sm font-medium tracking-[0.16em] text-accent uppercase">Setlist</p>
					<h1 class="mt-1 font-display text-display text-ink">{setlist.title}</h1>
					<p class="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-muted">
						<span>{formatServiceDate(setlist.service_date)}</span>
						{#if setlist.service_type}
							<span aria-hidden="true"> · </span>
							<span>{setlist.service_type}</span>
						{/if}
						<span aria-hidden="true"> · </span>
						<span>
							{items.length}
							{items.length === 1 ? 'item' : 'items'}
						</span>
						<span aria-hidden="true"> · </span>
						<span>{formatDuration(total)}</span>
					</p>
				</header>

				{#if items.length === 0}
					<p
						class="rounded-card border border-dashed border-border px-6 py-10 text-center text-muted"
					>
						This setlist has no items yet.
					</p>
				{:else}
					<ol class="flex flex-col gap-8">
						{#each items as item, index (item.id)}
							{@const chart = itemChart(item)}
							<li class="print-item border-b border-border pb-8 last:border-b-0 last:pb-0">
								<div class="flex items-baseline gap-3">
									<span class="w-7 shrink-0 text-right font-mono text-xs text-muted"
										>{String(index + 1).padStart(2, '0')}</span
									>
									<div class="min-w-0 flex-1">
										<div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
											<h2 class="font-display text-xl text-ink">
												{itemTitle(item)}
												{#if item.kind === 'song' && item.song?.artist}
													<span class="text-sm font-normal text-muted">· {item.song.artist}</span>
												{/if}
											</h2>
											<span class="text-sm text-muted">{formatDuration(itemDuration(item))}</span>
										</div>

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
										</div>

										{#if item.notes}
											<p class="mt-2 text-sm whitespace-pre-wrap text-muted">{item.notes}</p>
										{/if}

										{#if item.kind === 'song' && chart}
											{#if chart.capoFret > 0}
												<p class="mt-3 text-xs text-muted">
													Capo {chart.capoFret} · play in
													<span class="font-mono font-semibold text-accent"
														>{chart.shapeKey || '—'}</span
													>
												</p>
											{/if}
											<div class="print-chart mt-4 overflow-x-auto">
												<ChartView chart={chart.chart} />
											</div>
										{:else if item.kind === 'song'}
											<p class="mt-3 text-sm text-muted">
												The song behind this item is no longer available.
											</p>
										{/if}
									</div>
								</div>
							</li>
						{/each}
					</ol>
				{/if}

				<footer class="print-hide mt-10 border-t border-border pt-5 text-xs text-muted">
					Shared read-only from OpenSetlist. Changes to the setlist appear here automatically.
				</footer>
			</div>
		{/if}
	</main>
</div>
