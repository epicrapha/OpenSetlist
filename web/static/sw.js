// App-shell service worker (PWA).
//
// Strategy: network-first for navigations, cache-first for same-origin static
// assets, never cache `/api/`. On install, the full shell is discovered from
// the emitted `index.html` (hashed entry/CSS) plus the client entry's Vite
// `__vite__mapDeps` list (every route node/chunk it can dynamically import), so
// a cold-offline navigation to `/` renders the app instead of a blank page. No
// build plugin needed. Bump CACHE to invalidate the shell.
const CACHE = 'opensetlist-shell-v2';

const CORE = [
	'/',
	'/index.html',
	'/manifest.webmanifest',
	'/icon-192.png',
	'/icon-512.png',
	'/icon-maskable-512.png',
	'/apple-touch-icon.png'
];

// Absolute bundle URLs referenced by the shell HTML.
const HTML_ASSET = /\/_app\/immutable\/[^"'\s)]+\.(?:js|css)/g;
// Vite's dependency map inside the client entry, relative to it (`../nodes/...`).
const ENTRY_ASSET = /\.\.\/(?:nodes|chunks|assets)\/[A-Za-z0-9_.-]+\.(?:js|css)/g;
const APP_ENTRY = /\/_app\/immutable\/entry\/app\.[^"'\s)]+\.js/;

async function collectShellUrls() {
	const urls = new Set();

	const response = await fetch('/index.html', { cache: 'no-store' });
	if (!response.ok) return urls;

	const html = await response.text();
	for (const match of html.match(HTML_ASSET) || []) urls.add(match);

	const appPath = html.match(APP_ENTRY)?.[0];
	if (appPath) {
		const appUrl = new URL(appPath, self.location.origin);
		const appResponse = await fetch(appUrl, { cache: 'no-store' });
		if (appResponse.ok) {
			for (const dep of (await appResponse.text()).match(ENTRY_ASSET) || []) {
				urls.add(new URL(dep, appUrl).pathname);
			}
		}
	}

	return urls;
}

async function precache() {
	const cache = await caches.open(CACHE);

	// Add individually so one missing URL cannot fail the whole install.
	await Promise.allSettled(CORE.map((url) => cache.add(url)));

	try {
		const urls = await collectShellUrls();
		await Promise.allSettled([...urls].map((url) => cache.add(url)));
	} catch {
		// Shell discovery failed (e.g. dev); the core entries are still cached.
	}
}

async function offlineShell(request) {
	return (
		(await caches.match(request)) ||
		(await caches.match('/')) ||
		(await caches.match('/index.html')) ||
		Response.error()
	);
}

self.addEventListener('install', (event) => {
	event.waitUntil(precache().then(() => self.skipWaiting()));
});

self.addEventListener('activate', (event) => {
	event.waitUntil(
		caches
			.keys()
			.then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key))))
			.then(() => self.clients.claim())
	);
});

self.addEventListener('fetch', (event) => {
	const request = event.request;
	if (request.method !== 'GET') return;

	const url = new URL(request.url);
	if (url.origin !== self.location.origin) return;
	if (url.pathname.startsWith('/api/')) return;

	if (request.mode === 'navigate') {
		event.respondWith(fetch(request).catch(() => offlineShell(request)));
		return;
	}

	event.respondWith(
		caches.match(request).then(
			(cached) =>
				cached ||
				fetch(request).then((response) => {
					if (response.ok) {
						const clone = response.clone();
						caches.open(CACHE).then((cache) => cache.put(request, clone));
					}
					return response;
				})
		)
	);
});
