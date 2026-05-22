export async function sleep(ms: number): Promise<void> {
	await new Promise((resolve) => setTimeout(resolve, ms));
}

export function random(from: number = 0, to: number) {
	return Math.random() * (to - from) + from;
}
