<script lang="ts">
	import { resolve } from '$app/paths';
	import { ApiError, createInvite, listInvites } from '#lib/api';
	import type { Invite, InviteRole } from '#lib/api';
	import { session } from '#lib/session.svelte';
	import { inviteStatus } from '#lib/invite';
	import { copyText } from '#lib/clipboard';
	import Alert from '#lib/components/Alert.svelte';
	import Button from '#lib/components/Button.svelte';
	import Select from '#lib/components/Select.svelte';
	import TextField from '#lib/components/TextField.svelte';

	const roleOptions: { value: InviteRole; label: string }[] = [
		{ value: 'member', label: 'Member' },
		{ value: 'viewer', label: 'Viewer' },
		{ value: 'leader', label: 'Leader' },
		{ value: 'admin', label: 'Admin' }
	];

	let inviteEmail = $state('');
	let inviteRole = $state('member');
	let invites = $state<Invite[]>([]);
	let loading = $state(true);
	let loadError = $state('');
	let creating = $state(false);
	let createError = $state('');
	let createdNotice = $state('');
	let copiedId = $state<string | null>(null);
	let copyError = $state('');
	let copiedTimer: ReturnType<typeof setTimeout> | undefined;
	let createErrorBox = $state<HTMLElement>();

	const activeWorkspace = $derived(
		session.workspaces.find((workspace) => workspace.id === session.activeWorkspaceId) ?? null
	);
	const canManage = $derived(
		activeWorkspace?.role === 'owner' || activeWorkspace?.role === 'admin'
	);

	async function load() {
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId || !canManage) {
			loading = false;
			return;
		}
		loading = true;
		loadError = '';
		try {
			invites = await listInvites(workspaceId);
		} catch (cause) {
			loadError =
				cause instanceof ApiError && cause.status === 403
					? 'Only workspace owners and admins can see invites.'
					: cause instanceof ApiError
						? cause.message
						: 'Could not load invites.';
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		void session.activeWorkspaceId;
		void load();
	});

	$effect(() => {
		if (createError) createErrorBox?.focus();
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const workspaceId = session.activeWorkspaceId;
		if (!workspaceId) return;
		createError = '';
		createdNotice = '';
		creating = true;
		try {
			const created = await createInvite(workspaceId, {
				email: inviteEmail.trim(),
				role: inviteRole as InviteRole
			});
			invites = [created, ...invites];
			inviteEmail = '';
			inviteRole = 'member';
			createdNotice = 'Invite created. Copy the link and send it to them.';
		} catch (cause) {
			createError = cause instanceof ApiError ? cause.message : 'Could not create the invite.';
		} finally {
			creating = false;
		}
	}

	async function copy(invite: Invite) {
		copyError = '';
		const ok = await copyText(`${window.location.origin}/invite/${invite.token}`);
		if (!ok) {
			copyError = 'Could not copy the link. Open the invite page and copy it from the address bar.';
			return;
		}
		copiedId = invite.id;
		clearTimeout(copiedTimer);
		copiedTimer = setTimeout(() => {
			if (copiedId === invite.id) copiedId = null;
		}, 2000);
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString(undefined, {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		});
	}
</script>

{#if !session.activeWorkspaceId}
	<div class="max-w-lg">
		<h1 class="font-display text-display text-ink">Invites</h1>
		<p class="mt-2 text-muted">Choose a workspace before you manage invites.</p>
		<a
			href={resolve('/app')}
			class="mt-6 inline-flex min-h-11 items-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
		>
			Choose a workspace
		</a>
	</div>
{:else if !canManage}
	<header class="mb-8">
		<h1 class="font-display text-display text-ink">Invites</h1>
		<p class="mt-2 max-w-prose text-muted">
			Manage who can join {activeWorkspace?.name ?? 'this workspace'}.
		</p>
	</header>
	<div class="max-w-lg">
		<Alert variant="info">
			Only workspace owners and admins can manage invites. Ask one of them to invite someone.
		</Alert>
	</div>
{:else}
	<header class="mb-8">
		<h1 class="font-display text-display text-ink">Invites</h1>
		<p class="mt-2 max-w-prose text-muted">
			Invite people to {activeWorkspace?.name ?? 'this workspace'}. They join with the link and the
			role you choose.
		</p>
	</header>

	<div class="grid gap-8 lg:grid-cols-[1.5fr_1fr] lg:items-start">
		<section aria-labelledby="invites-heading">
			<h2 id="invites-heading" class="sr-only">Invites</h2>

			{#if loading}
				<div role="status" class="flex items-center gap-3 py-8 text-muted">
					<span
						class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
						aria-hidden="true"
					></span>
					Loading invites…
				</div>
			{:else if loadError}
				<div class="max-w-lg">
					<Alert>{loadError}</Alert>
					<Button class="mt-4" variant="secondary" onclick={load}>Try again</Button>
				</div>
			{:else if invites.length === 0}
				<div class="rounded-card border border-dashed border-border px-6 py-10 text-center">
					<h3 class="font-display text-lg text-ink">No invites yet</h3>
					<p class="mt-1 text-sm text-muted">
						Create one and send the link to the person you want to add.
					</p>
				</div>
			{:else}
				<ul class="flex flex-col gap-3">
					{#each invites as invite (invite.id)}
						{@const status = inviteStatus(invite)}
						<li
							class="flex flex-wrap items-center justify-between gap-3 rounded-card border border-border bg-surface px-4 py-3 sm:px-5"
						>
							<div class="min-w-0">
								<p class="truncate font-medium text-ink">{invite.email}</p>
								<p class="mt-0.5 text-sm text-muted">
									<span class="capitalize">{invite.role}</span>
									<span aria-hidden="true"> · </span>
									{#if status === 'pending'}
										Expires {formatDate(invite.expires_at)}
									{:else if status === 'accepted'}
										Accepted {formatDate(invite.accepted_at ?? invite.created_at)}
									{:else}
										Expired {formatDate(invite.expires_at)}
									{/if}
								</p>
							</div>
							<div class="flex shrink-0 items-center gap-2">
								<span
									class="rounded-full px-2.5 py-0.5 text-xs font-semibold {status === 'pending'
										? 'bg-accent-tint text-accent'
										: status === 'accepted'
											? 'bg-success-tint text-success-text'
											: 'bg-surface-2 text-muted'}"
								>
									{status === 'pending'
										? 'Pending'
										: status === 'accepted'
											? 'Accepted'
											: 'Expired'}
								</span>
								{#if status === 'pending'}
									<Button
										variant="secondary"
										aria-label="Copy invite link for {invite.email}"
										onclick={() => copy(invite)}
									>
										{copiedId === invite.id ? 'Copied' : 'Copy link'}
									</Button>
								{/if}
							</div>
						</li>
					{/each}
				</ul>
			{/if}

			<p role="status" class="sr-only">{copiedId ? 'Invite link copied.' : ''}</p>
			{#if copyError}
				<div class="mt-4 max-w-lg"><Alert>{copyError}</Alert></div>
			{/if}
		</section>

		<section
			aria-labelledby="create-invite-heading"
			class="rounded-card border border-border bg-surface p-6"
		>
			<h2 id="create-invite-heading" class="font-display text-xl text-ink">Invite someone</h2>
			<p class="mt-1 text-sm text-muted">They'll get access as soon as they open the link.</p>

			<form class="mt-5 flex flex-col gap-4" onsubmit={submit}>
				<TextField
					label="Email"
					name="invite-email"
					type="email"
					autocomplete="email"
					placeholder="name@example.com"
					required
					bind:value={inviteEmail}
				/>
				<Select
					label="Role"
					name="invite-role"
					bind:value={inviteRole}
					hint="Leaders and above can edit songs."
				>
					{#each roleOptions as option (option.value)}
						<option value={option.value}>{option.label}</option>
					{/each}
				</Select>

				{#if createError}
					<div bind:this={createErrorBox} tabindex="-1" class="rounded-field outline-none">
						<Alert>{createError}</Alert>
					</div>
				{/if}
				{#if createdNotice}
					<Alert variant="success">{createdNotice}</Alert>
				{/if}

				<Button type="submit" block loading={creating}>
					{creating ? 'Creating…' : 'Create invite'}
				</Button>
			</form>
		</section>
	</div>
{/if}
