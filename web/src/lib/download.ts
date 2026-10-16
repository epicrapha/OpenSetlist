/** Save a fetched response to disk, honouring a Content-Disposition filename. */
export async function saveResponse(response: Response, fallbackName: string): Promise<void> {
	const blob = await response.blob();
	const disposition = response.headers.get('content-disposition') ?? '';
	const match = /filename="?([^";]+)"?/.exec(disposition);
	const name = match?.[1]?.trim() || fallbackName;

	const url = URL.createObjectURL(blob);
	const anchor = document.createElement('a');
	anchor.href = url;
	anchor.download = name;
	document.body.appendChild(anchor);
	anchor.click();
	anchor.remove();
	URL.revokeObjectURL(url);
}

/** Human-readable file size, e.g. `1.2 MB`. */
export function formatBytes(bytes: number | null | undefined): string {
	if (bytes === null || bytes === undefined) return '';
	if (bytes < 1024) return `${bytes} B`;
	const units = ['KB', 'MB', 'GB'];
	let value = bytes / 1024;
	let unit = 0;
	while (value >= 1024 && unit < units.length - 1) {
		value /= 1024;
		unit += 1;
	}
	return `${value.toFixed(1)} ${units[unit]}`;
}
