<script lang="ts">
	import { resolve } from '$app/paths';
	import Brand from '../lib/components/Brand.svelte';
	import { buttonClasses } from '../lib/components/button';

	const signUp = resolve('/register');
	const signIn = resolve('/login');

	const preview = [
		{ title: '10,000 Reasons', key: 'G' },
		{ title: 'Great Are You Lord', key: 'A' },
		{ title: 'How Great Is Our God', key: 'C' },
		{ title: 'Goodness of God', key: 'Ab' }
	];

	const features = [
		{
			title: 'Your whole library',
			body: 'Keep every song’s chords, key, and notes in one place, ready to drop into any set.'
		},
		{
			title: 'Setlists you can share',
			body: 'Build the order for Sunday, set the key for each song, and send one link to the band.'
		},
		{
			title: 'Made for the stage',
			body: 'Large type, strong contrast, and big touch targets, so it stays readable in a dim room.'
		}
	];
</script>

<div class="flex min-h-dvh flex-col">
	<header
		class="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-between gap-x-4 gap-y-3 px-5 py-5 sm:px-8"
	>
		<Brand />
		<nav aria-label="Primary" class="flex items-center gap-2">
			<a class={buttonClasses('ghost')} href={signIn}>Sign in</a>
			<a class={buttonClasses('primary')} href={signUp}>Create account</a>
		</nav>
	</header>

	<main id="main" class="mx-auto w-full max-w-6xl flex-1 px-5 sm:px-8">
		<section class="grid items-center gap-12 py-12 lg:grid-cols-[1.05fr_0.95fr] lg:py-20">
			<div class="animate-rise">
				<h1 class="max-w-2xl font-display text-hero text-balance text-ink">
					Setlists built for the stage.
				</h1>
				<p class="mt-6 max-w-xl text-lg text-pretty text-muted">
					A calm, readable home for your team’s songs, keys, and setlists. Big enough to read from
					the back of the room, simple enough to use on the way in.
				</p>
				<div class="mt-8 flex flex-wrap items-center gap-3">
					<a class={buttonClasses('primary', 'lg')} href={signUp}>Create an account</a>
					<a class={buttonClasses('secondary', 'lg')} href={signIn}>Sign in</a>
				</div>
			</div>

			<div class="animate-rise" style="animation-delay: 120ms" aria-hidden="true">
				<div class="rounded-card border border-border bg-surface p-5 shadow-lift">
					<div class="flex items-center justify-between gap-4">
						<p class="font-display text-xl text-ink">Sunday morning</p>
						<span class="rounded-full bg-accent-tint px-3 py-1 text-xs font-semibold text-accent">
							4 songs
						</span>
					</div>
					<ol class="mt-4 divide-y divide-border">
						{#each preview as song, index (song.title)}
							<li class="flex items-center justify-between gap-4 py-3">
								<span class="flex min-w-0 items-center gap-3">
									<span class="w-4 shrink-0 text-sm text-muted tabular-nums">{index + 1}</span>
									<span class="truncate text-ink">{song.title}</span>
								</span>
								<span class="shrink-0 font-mono text-sm text-accent">{song.key}</span>
							</li>
						{/each}
					</ol>
				</div>
			</div>
		</section>

		<section aria-labelledby="features-heading" class="border-t border-border py-12 lg:py-16">
			<h2 id="features-heading" class="sr-only">What OpenSetlist does</h2>
			<ul class="grid gap-x-8 gap-y-10 sm:grid-cols-3">
				{#each features as feature (feature.title)}
					<li class="border-t border-border pt-5">
						<h3 class="font-display text-lg text-ink">{feature.title}</h3>
						<p class="mt-2 text-sm text-pretty text-muted">{feature.body}</p>
					</li>
				{/each}
			</ul>
		</section>
	</main>

	<footer class="border-t border-border">
		<div
			class="mx-auto flex w-full max-w-6xl flex-wrap items-center justify-between gap-2 px-5 py-6 text-sm text-muted sm:px-8"
		>
			<p>OpenSetlist — setlists for worship teams.</p>
			<p>Open source under AGPL-3.0.</p>
		</div>
	</footer>
</div>
