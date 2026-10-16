<script lang="ts">
	import type { Chart } from '#lib/chord';

	let { chart, class: className = '' }: { chart: Chart; class?: string } = $props();
</script>

<div class="flex flex-col gap-7 {className}">
	{#each chart.sections as section, si (si)}
		<section class="flex flex-col gap-2.5">
			{#if section.label}
				<h3
					class="flex items-center gap-2 text-xs font-semibold tracking-[0.14em] text-muted uppercase"
				>
					<span class="h-px w-5 bg-border-strong" aria-hidden="true"></span>
					{section.label}
				</h3>
			{/if}
			<div class="flex flex-col gap-1.5">
				{#each section.lines as line, li (li)}
					{#if line.kind === 'empty'}
						<div class="h-2" aria-hidden="true"></div>
					{:else if line.kind === 'lyric'}
						<p class="font-mono text-sm whitespace-pre-wrap text-ink/90">{line.text}</p>
					{:else}
						<p class="flex flex-wrap items-end font-mono text-sm">
							{#each line.segments as segment, gi (gi)}
								<span class="inline-flex flex-col">
									<span class="h-5 font-semibold text-accent">{segment.chord ?? ''}</span>
									<span class="whitespace-pre text-ink">{segment.lyric}</span>
								</span>
							{/each}
						</p>
					{/if}
				{/each}
			</div>
		</section>
	{/each}
</div>
