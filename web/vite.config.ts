import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			// SPA: emit an index.html fallback for all un-prerendered routes.
			adapter: adapter({ fallback: 'index.html' })
		})
	],
	server: {
		// Dev-only: same-origin API calls (`PUBLIC_API_BASE_URL` empty) hit the Rust server.
		proxy: {
			'/api': { target: 'http://localhost:8080', changeOrigin: true }
		}
	},
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'node'
	}
});
