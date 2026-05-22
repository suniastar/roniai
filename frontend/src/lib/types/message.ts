import { decode, decodeAsync } from '@msgpack/msgpack';
import { ArrayQueue, ConstantBackoff, WebsocketBuilder } from 'websocket-ts';

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

export async function playbackAudio(context: AudioContext | null, audio: Audio): Promise<void> {
	if (!context) {
		return;
	}
	return new Promise((resolve) => {
		let buffer = context.createBuffer(1, audio.samples.length, audio.sample_rate);
		buffer.copyToChannel(new Float32Array(audio.samples), 0);
		const source = context.createBufferSource();
		source.buffer = buffer;
		source.connect(context.destination);
		source.onended = () => resolve();
		source.start();
	});
}
