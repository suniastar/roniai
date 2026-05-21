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

export async function playbackAudio(context: AudioContext, message: Message): Promise<void> {
	if (message.type !== 'say') {
		return;
	}
	let buffer = context.createBuffer(1, message.req_wav.samples.length, message.req_wav.sample_rate);
	buffer.copyToChannel(new Float32Array(message.req_wav.samples), 0);
	const source = context.createBufferSource();
	source.buffer = buffer;
	source.connect(context.destination);
	source.start();
}

function test() {
	const ws = new WebsocketBuilder('ws://localhost:8080/ws/mrsroni')
		.withInstantReconnect(true)
		.withBuffer(new ArrayQueue())
		.withBackoff(new ConstantBackoff(1000))
		.onOpen((i, ev) => {
			console.log('opened', i, ev);
		})
		.onClose((i, ev) => {
			console.log('closed', i, ev);
		})
		.onError((i, ev) => {
			console.log('error', i, ev);
		})
		.onMessage(async (i, ev) => {
			let bytes = ev.data;
			let msg = await decodeMessage(bytes);
			console.log('message', msg);
		})
		.onRetry((i, ev) => {
			console.log('retry', i, ev);
		})
		.onReconnect((i, ev) => {
			console.log('reconnect', i, ev);
		})
		.build();
}
