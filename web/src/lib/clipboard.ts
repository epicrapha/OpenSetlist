/** Copy text, falling back to the legacy path when the Clipboard API is unavailable. */
export async function copyText(text: string): Promise<boolean> {
	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		try {
			const field = document.createElement('textarea');
			field.value = text;
			field.setAttribute('readonly', '');
			field.style.position = 'fixed';
			field.style.opacity = '0';
			document.body.appendChild(field);
			field.select();
			const ok = document.execCommand('copy');
			field.remove();
			return ok;
		} catch {
			return false;
		}
	}
}
