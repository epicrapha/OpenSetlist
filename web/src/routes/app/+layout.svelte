<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { logout } from '../../lib/api';
	import { loadSession, resetSession, session } from '../../lib/session.svelte';
	import Alert from '../../lib/components/Alert.svelte';
	import Brand from '../../lib/components/Brand.svelte';
	import Button from '../../lib/components/Button.svelte';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);

	let reloading = $state(false);

	onMount(() => {
		void loadSession();
	});

	async function retry() {
		reloading = true;
		await loadSession(true);
		reloading = false;
	}

	async function signOut() {
		try {
			await logout();
		} catch {
			// session already gone; nothing to do.
		}
		resetSession();
		await goto(resolve('/login'));
	}
</script>

<div class="flex min-h-dvh flex-col">
	<header class="sticky top-0 z-40 border-b border-border bg-canvas/85 backdrop-blur">
		<div class="mx-auto flex h-16 w-full max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
			<div class="flex min-w-0 items-center gap-3">
				<Brand />
				{#if activeWorkspace}
					<span class="hidden h-5 w-px shrink-0 bg-border sm:block" aria-hidden="true"></span>
					<span class="hidden truncate text-sm text-muted sm:block">{activeWorkspace.name}</span>
				{/if}
			</div>
			<div class="flex shrink-0 items-center gap-3">
				{#if session.user}
					<span class="hidden text-sm text-muted md:block">{session.user.name}</span>
				{/if}
				<Button variant="secondary" onclick={signOut}>Sign out</Button>
			</div>
		</div>
	</header>

	<main id="main" class="mx-auto w-full max-w-6xl flex-1 px-4 py-8 sm:px-6 sm:py-10">
		{#if session.status === 'idle' || session.status === 'loading'}
			<div role="status" class="flex items-center gap-3 py-10 text-muted">
				<span
					class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
					aria-hidden="true"
				></span>
				Loading your account…
			</div>
		{:else if session.status === 'error'}
			<div class="max-w-lg py-4">
				<Alert>{session.error}</Alert>
				<Button class="mt-4" variant="secondary" loading={reloading} onclick={retry}
					>Try again</Button
				>
			</div>
		{:else}
			{@render children()}
		{/if}
	</main>
</div>
