<script lang="ts">
	import { resolve } from '$app/paths';
</script>

<a
	href={resolve('/')}
	class="inline-flex items-center gap-2.5 rounded-lg text-ink focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
>
	<span
		class="grid size-9 shrink-0 place-items-center rounded-[0.6rem] bg-accent text-accent-ink"
		aria-hidden="true"
	>
		<svg viewBox="0 0 24 24" fill="none" class="size-5">
			<path
				d="M5 8h9M5 12h11M5 16h6"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
			/>
			<circle cx="18" cy="7.5" r="2.2" fill="currentColor" />
		</svg>
	</span>
	<span class="text-lg font-semibold tracking-tight">OpenSetlist</span>
</a>
