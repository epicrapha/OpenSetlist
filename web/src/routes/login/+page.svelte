<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { ApiError, login } from '../../lib/api';
	import { safeNext } from '../../lib/safeNext';
	import Alert from '../../lib/components/Alert.svelte';
	import Brand from '../../lib/components/Brand.svelte';
	import Button from '../../lib/components/Button.svelte';
	import TextField from '../../lib/components/TextField.svelte';

	const next = $derived(page.url.searchParams.get('next'));
	const registerHref = $derived(
		resolve(next ? `/register?next=${encodeURIComponent(next)}` : '/register')
	);

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let submitting = $state(false);
	let errorBox = $state<HTMLElement>();

	// Move focus to the error so keyboard and screen-reader users land on it (WCAG 3.3.1).
	$effect(() => {
		if (error) errorBox?.focus();
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		error = '';
		submitting = true;
		try {
			await login({ email, password });
			await goto(safeNext(next, resolve('/app')));
		} catch (cause) {
			error = cause instanceof ApiError ? cause.message : 'Something went wrong. Please try again.';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="flex min-h-dvh flex-col">
	<header class="px-5 py-5 sm:px-8">
		<Brand />
	</header>

	<main id="main" class="flex flex-1 items-start justify-center px-5 pt-6 pb-20 sm:pt-12">
		<div class="w-full max-w-sm">
			<h1 class="font-display text-display text-ink">Sign in</h1>
			<p class="mt-2 text-muted">Welcome back. Sign in to open your setlists.</p>

			<form class="mt-8 flex flex-col gap-4" onsubmit={handleSubmit}>
				<TextField
					label="Email"
					name="email"
					type="email"
					autocomplete="email"
					required
					bind:value={email}
				/>
				<TextField
					label="Password"
					name="password"
					type="password"
					autocomplete="current-password"
					required
					bind:value={password}
				/>

				{#if error}
					<div bind:this={errorBox} tabindex="-1" class="rounded-field outline-none">
						<Alert>{error}</Alert>
					</div>
				{/if}

				<Button type="submit" size="lg" block loading={submitting}>
					{submitting ? 'Signing in…' : 'Sign in'}
				</Button>
			</form>

			<p class="mt-6 text-center text-sm text-muted">
				New here?
				<a class="font-medium text-accent hover:underline" href={registerHref}>
					Create an account
				</a>
			</p>
		</div>
	</main>
</div>
