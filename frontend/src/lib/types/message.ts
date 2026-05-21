import { decode, decodeAsync } from '@msgpack/msgpack';

export type Message =
	| {
			type: 'eval';
			id: number;
			req_txt: string;
	  }
	| {
			type: 'say';
			id: number;
			req_wav: Audio;
			res_txt: string;
			res_wav: Audio;
	  };

export type Audio = {
	samples: number[];
	sample_rate: number;
};

export async function decodeMessage(blob: Blob): Promise<Message> {
	const stream = blob.stream();
	if (stream) {
		return (await decodeAsync(stream)) as Message;
	} else {
		return decode(await blob.arrayBuffer()) as Message;
	}
}
