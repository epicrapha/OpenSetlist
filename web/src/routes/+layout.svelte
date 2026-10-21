<script lang="ts">
	import './layout.css';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { dev } from '$app/env';
	import { onUnauthorized } from '../lib/api';
	import favicon from '../lib/assets/favicon.svg';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	onMount(() => {
		// Central 401 handling: any authenticated request that loses its session
		// sends the user back to sign-in.
		const unsubscribe = onUnauthorized(() => {
			void goto(resolve('/login'));
		});

		if (!dev && 'serviceWorker' in navigator) {
			navigator.serviceWorker.register('/sw.js').catch((error) => {
				console.warn('Service worker registration failed', error);
			});
		}

		return unsubscribe;
	});
</script>

<svelte:head>
	<!-- Only the dynamic favicon import goes here; the static PWA tags live in
	     src/app.html so they land in the prerendered index.html. -->
	<link rel="icon" href={favicon} />
</svelte:head>

<a
	href="#main"
	class="print-hide sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:rounded-field focus:bg-accent focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:text-accent-ink"
>
	Skip to content
</a>
{@render children()}
