<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { logout } from '#lib/api';
	import { loadSession, resetSession, session, setActiveWorkspace } from '#lib/session.svelte';
	import Alert from '#lib/components/Alert.svelte';
	import Brand from '#lib/components/Brand.svelte';
	import Button from '#lib/components/Button.svelte';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);

	const navItems = $derived([
		{ href: resolve('/app/setlists'), label: 'Setlists' },
		{ href: resolve('/app/songs'), label: 'Songs' },
		...(activeWorkspace?.role === 'owner' || activeWorkspace?.role === 'admin'
			? [{ href: resolve('/app/invites'), label: 'Invites' }]
			: [])
	]);
	const currentPath = $derived(page.url.pathname);

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
	<header class="print-hide sticky top-0 z-40 border-b border-border bg-canvas/85 backdrop-blur">
		<div class="mx-auto flex h-16 w-full max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
			<div class="flex min-w-0 items-center gap-4">
				<Brand />
				{#if session.status === 'ready'}
					<span class="hidden h-5 w-px shrink-0 bg-border sm:block" aria-hidden="true"></span>
					<nav aria-label="Main" class="flex items-center gap-1">
						{#each navItems as item (item.href)}
							{@const active = currentPath === item.href || currentPath.startsWith(`${item.href}/`)}
							<a
								href={item.href}
								aria-current={active ? 'page' : undefined}
								class="rounded-lg px-2.5 py-1.5 text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {active
									? 'bg-surface-2 text-ink'
									: 'text-muted hover:bg-surface-2 hover:text-ink'}"
							>
								{item.label}
							</a>
						{/each}
					</nav>
				{/if}
			</div>
			<div class="flex shrink-0 items-center gap-3">
				{#if session.status === 'ready' && activeWorkspace}
					{#if session.workspaces.length > 1}
						<label class="sr-only" for="workspace-switcher">Workspace</label>
						<select
							id="workspace-switcher"
							value={activeWorkspace.id}
							onchange={(event) => setActiveWorkspace(event.currentTarget.value)}
							class="hidden h-9 max-w-40 rounded-field bg-surface px-2.5 text-sm text-ink ring-1 ring-border transition-colors focus:ring-2 focus:ring-accent focus:outline-none sm:block"
						>
							{#each session.workspaces as workspace (workspace.id)}
								<option value={workspace.id}>{workspace.name}</option>
							{/each}
						</select>
					{:else}
						<span class="hidden max-w-40 truncate text-sm text-muted sm:block"
							>{activeWorkspace.name}</span
						>
					{/if}
				{/if}
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
