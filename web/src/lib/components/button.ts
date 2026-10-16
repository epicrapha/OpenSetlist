export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'md' | 'lg';

const BASE =
	'inline-flex items-center justify-center gap-2 rounded-field font-medium whitespace-nowrap transition-colors duration-150 select-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent active:translate-y-px disabled:cursor-not-allowed disabled:opacity-55 disabled:active:translate-y-0';

const SIZES: Record<ButtonSize, string> = {
	md: 'min-h-11 px-4 text-[0.9375rem]',
	lg: 'min-h-13 px-6 text-base'
};

const VARIANTS: Record<ButtonVariant, string> = {
	primary: 'bg-accent text-accent-ink hover:bg-accent-hover',
	secondary: 'bg-surface-2 text-ink ring-1 ring-border hover:bg-surface hover:ring-border-strong',
	ghost: 'text-muted hover:bg-surface-2 hover:text-ink',
	danger: 'bg-danger text-canvas hover:bg-danger-hover'
};

/** Shared button styling, so links can look like buttons too. */
export function buttonClasses(
	variant: ButtonVariant = 'primary',
	size: ButtonSize = 'md',
	block = false,
	extra = ''
): string {
	return [BASE, SIZES[size], VARIANTS[variant], block ? 'w-full' : '', extra]
		.filter(Boolean)
		.join(' ');
}
