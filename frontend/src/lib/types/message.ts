import { decode, decodeAsync } from '@msgpack/msgpack';

export type Message =
	| {
			type: 'eval';
			id: number;
			prompt: string;
	  }
	| {
			type: 'say';
			id: number;
			prompt: string;
	  };

export async function decodeMessage(blob: Blob): Promise<Message> {
	const stream = blob.stream();
	if (stream) {
		return (await decodeAsync(stream)) as Message;
	} else {
		return decode(await blob.arrayBuffer()) as Message;
	}
}
