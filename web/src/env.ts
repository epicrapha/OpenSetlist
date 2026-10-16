import { defineEnvVars } from '@sveltejs/kit/env';

// SvelteKit 3 env API. Public vars are importable from `$app/env/public`.
export const variables = defineEnvVars({
	// Empty = same-origin (dev proxy / nginx proxy).
	PUBLIC_API_BASE_URL: {
		public: true,
		static: true,
		schema: (value) => value ?? ''
	}
});
