<script lang="ts">
	import { ApiError, createWorkspace } from '../../lib/api';
	import { loadSession, session, setActiveWorkspace } from '../../lib/session.svelte';
	import Alert from '../../lib/components/Alert.svelte';
	import Button from '../../lib/components/Button.svelte';
	import TextField from '../../lib/components/TextField.svelte';

	let newName = $state('');
	let creating = $state(false);
	let createError = $state('');

	async function handleCreate(event: SubmitEvent) {
		event.preventDefault();
		createError = '';
		creating = true;
		try {
			const { workspace } = await createWorkspace(newName.trim());
			newName = '';
			await loadSession(true);
			setActiveWorkspace(workspace.id);
		} catch (cause) {
			createError = cause instanceof ApiError ? cause.message : 'Could not create workspace.';
		} finally {
			creating = false;
		}
	}
</script>

<header class="mb-8">
	<h1 class="font-display text-display text-ink">Workspaces</h1>
	<p class="mt-2 max-w-prose text-muted">
		A workspace holds your team’s songs and setlists. Pick one to work in, or create a new one.
	</p>
</header>

<div class="grid gap-8 lg:grid-cols-[1.4fr_1fr] lg:items-start">
	<section aria-labelledby="workspaces-heading">
		<h2 id="workspaces-heading" class="sr-only">Your workspaces</h2>

		{#if session.workspaces.length === 0}
			<div class="rounded-card border border-dashed border-border px-6 py-12 text-center">
				<span
					class="mx-auto grid size-12 place-items-center rounded-full bg-surface-2 text-muted"
					aria-hidden="true"
				>
					<svg viewBox="0 0 24 24" fill="none" class="size-6">
						<path
							d="M4 7.5 12 3l8 4.5v9L12 21l-8-4.5v-9Z"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linejoin="round"
						/>
						<path
							d="M4 7.5 12 12l8-4.5M12 12v9"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linejoin="round"
						/>
					</svg>
				</span>
				<h3 class="mt-4 font-display text-lg text-ink">No workspaces yet</h3>
				<p class="mt-1 text-sm text-muted">Create your first one to start building setlists.</p>
			</div>
		{:else}
			<ul class="flex flex-col gap-3">
				{#each session.workspaces as workspace (workspace.id)}
					{@const active = workspace.id === session.activeWorkspaceId}
					<li>
						<button
							type="button"
							aria-pressed={active}
							onclick={() => setActiveWorkspace(workspace.id)}
							class="group flex w-full items-center justify-between gap-4 rounded-card border px-5 py-4 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent {active
								? 'border-accent bg-accent-tint'
								: 'border-border bg-surface hover:border-border-strong hover:bg-surface-2'}"
						>
							<span class="min-w-0">
								<span class="block truncate text-lg font-medium text-ink">{workspace.name}</span>
								<span class="mt-0.5 block text-sm text-muted capitalize">{workspace.role}</span>
							</span>
							{#if active}
								<span
									class="shrink-0 rounded-full bg-accent px-3 py-1 text-xs font-semibold text-accent-ink"
								>
									Active
								</span>
							{:else}
								<span class="shrink-0 text-sm text-muted group-hover:text-ink">Select</span>
							{/if}
						</button>
					</li>
				{/each}
			</ul>
			<p class="mt-4 text-sm text-muted">Your choice is remembered on this device.</p>
		{/if}
	</section>

	<section
		aria-labelledby="create-heading"
		class="rounded-card border border-border bg-surface p-6"
	>
		<h2 id="create-heading" class="font-display text-xl text-ink">Create a workspace</h2>
		<p class="mt-1 text-sm text-muted">Name it after your church or worship band.</p>

		<form class="mt-5 flex flex-col gap-4" onsubmit={handleCreate}>
			<TextField
				label="Workspace name"
				name="name"
				placeholder="Grace Community Worship"
				required
				bind:value={newName}
			/>

			{#if createError}
				<Alert>{createError}</Alert>
			{/if}

			<Button type="submit" block loading={creating}>
				{creating ? 'Creating…' : 'Create workspace'}
			</Button>
		</form>
	</section>
</div>
