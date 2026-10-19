<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ApiError, acceptInvite, me } from '#lib/api';
	import type { AcceptInviteResult } from '#lib/api';
	import { acceptErrorKind, invitePath, type AcceptErrorKind } from '#lib/invite';
	import { loadSession, setActiveWorkspace } from '#lib/session.svelte';
	import Alert from '#lib/components/Alert.svelte';
	import Brand from '#lib/components/Brand.svelte';
	import Button from '#lib/components/Button.svelte';
	import type { PageProps } from './$types';

	type Phase = 'checking' | 'needs-signin' | 'accepting' | 'success' | 'error';

	let { params }: PageProps = $props();
	const token = $derived(params.token);

	let phase = $state<Phase>('checking');
	let result = $state<AcceptInviteResult | null>(null);
	let errorKind = $state<AcceptErrorKind>('other');
	let errorBox = $state<HTMLElement>();
	let lastToken = '';

	const nextPath = $derived(invitePath(token));
	const loginHref = $derived(resolve(`/login?next=${encodeURIComponent(nextPath)}`));
	const registerHref = $derived(resolve(`/register?next=${encodeURIComponent(nextPath)}`));

	const errorCopy: Record<AcceptErrorKind, { title: string; body: string }> = {
		unknown: {
			title: 'Invite not found',
			body: 'This link doesn’t match an invite. Check that you opened the whole link, or ask the workspace owner to send a new one.'
		},
		expired: {
			title: 'Invite expired',
			body: 'This invite is past its expiry date. Ask the workspace owner for a fresh one.'
		},
		accepted: {
			title: 'Invite already used',
			body: 'This invite has already been accepted. If you think that’s a mistake, ask the workspace owner to check.'
		},
		member: {
			title: 'You’re already a member',
			body: 'You already have access to this workspace, so there’s nothing more to accept.'
		},
		'wrong-email': {
			title: 'This invite is for a different email',
			body: 'The invite was sent to a different address. Sign in with that email address to accept it.'
		},
		'signed-out': {
			title: 'Please sign in',
			body: 'Sign in to accept this invite.'
		},
		other: {
			title: 'Couldn’t accept the invite',
			body: 'Something went wrong on our side. Please try again in a moment.'
		}
	};

	$effect(() => {
		if (phase === 'error') errorBox?.focus();
	});

	async function run() {
		if (!token) {
			phase = 'error';
			errorKind = 'unknown';
			return;
		}
		phase = 'checking';
		result = null;
		try {
			// Probe without the central 401 redirect: signed-out visitors should see
			// this page, not get bounced away from their invite.
			await me({ auth: false });
		} catch (cause) {
			if (cause instanceof ApiError && cause.status === 401) {
				phase = 'needs-signin';
			} else {
				phase = 'error';
				errorKind = 'other';
			}
			return;
		}

		phase = 'accepting';
		try {
			result = await acceptInvite(token);
			phase = 'success';
		} catch (cause) {
			phase = 'error';
			errorKind = acceptErrorKind(cause);
		}
	}

	$effect(() => {
		// Run once per token; also handles a token change without a remount.
		if (token === lastToken) return;
		lastToken = token;
		void run();
	});

	async function openWorkspace() {
		if (!result) return;
		setActiveWorkspace(result.workspace_id);
		await loadSession(true);
		await goto(resolve('/app/songs'));
	}

	function retry() {
		void run();
	}
</script>

<div class="flex min-h-dvh flex-col">
	<header class="px-5 py-5 sm:px-8">
		<Brand />
	</header>

	<main id="main" class="flex flex-1 items-start justify-center px-5 pt-6 pb-20 sm:pt-12">
		<div class="w-full max-w-md">
			<div class="rounded-card border border-border bg-surface p-6 shadow-lift sm:p-8">
				{#if phase === 'checking' || phase === 'accepting'}
					<div role="status" class="flex items-center gap-3 py-4 text-muted">
						<span
							class="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
							aria-hidden="true"
						></span>
						{phase === 'checking' ? 'Checking your session…' : 'Accepting your invite…'}
					</div>
				{:else if phase === 'needs-signin'}
					<h1 class="font-display text-display text-ink">You’re invited</h1>
					<p class="mt-3 text-muted">
						Sign in or create an account to accept this invite. We’ll bring you right back here.
					</p>
					<div class="mt-6 flex flex-col gap-3">
						<a
							href={loginHref}
							class="inline-flex min-h-11 items-center justify-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
						>
							Sign in to accept
						</a>
						<a
							href={registerHref}
							class="inline-flex min-h-11 items-center justify-center rounded-field bg-surface-2 px-4 font-medium text-ink ring-1 ring-border transition-colors hover:bg-surface"
						>
							Create an account
						</a>
					</div>
				{:else if phase === 'success' && result}
					<h1 class="font-display text-display text-ink">You’re in</h1>
					<p class="mt-3 text-muted">
						You joined the workspace as a <span class="font-medium text-ink capitalize"
							>{result.role}</span
						>. You can start browsing songs right away.
					</p>
					<Button class="mt-6" block size="lg" onclick={openWorkspace}>Open workspace</Button>
				{:else}
					<div bind:this={errorBox} tabindex="-1" class="outline-none">
						<h1 class="font-display text-display text-ink">{errorCopy[errorKind].title}</h1>
						<p class="mt-3 text-muted">{errorCopy[errorKind].body}</p>
						{#if errorKind === 'wrong-email' || errorKind === 'signed-out'}
							<Alert variant="info" class="mt-4">
								Signed in as someone else? Sign in with the invited address.
							</Alert>
							<a
								href={loginHref}
								class="mt-4 inline-flex min-h-11 items-center justify-center rounded-field bg-accent px-4 font-medium text-accent-ink transition-colors hover:bg-accent-hover"
							>
								Sign in with another account
							</a>
						{:else if errorKind === 'accepted' || errorKind === 'member'}
							<a
								href={resolve('/app')}
								class="mt-5 inline-flex min-h-11 items-center rounded-field bg-surface-2 px-4 font-medium text-ink ring-1 ring-border transition-colors hover:bg-surface"
							>
								Go to your workspaces
							</a>
						{:else if errorKind === 'other'}
							<Button class="mt-5" variant="secondary" onclick={retry}>Try again</Button>
						{:else}
							<a
								href={resolve('/')}
								class="mt-5 inline-flex text-sm font-medium text-accent hover:underline"
							>
								Back to OpenSetlist
							</a>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</main>
</div>
