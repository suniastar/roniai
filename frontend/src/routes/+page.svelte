<script lang="ts">
	import { random, sleep } from '$lib/roni';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { ArrayQueue, ConstantBackoff, WebsocketBuilder } from 'websocket-ts';
	import { decodeMessage, playbackAudio } from '$lib/types/message';
	import { PUBLIC_WEBSOCKET_URL } from '$env/static/public';
	import Roni from '$lib/components/roni.svelte';
	import check from '$lib/icons/check.svg';
	import rotate from '$lib/icons/rotate.svg';
	import triangle from '$lib/icons/triangle_exclamation.svg';
	import xmark from '$lib/icons/xmark.svg';

	let audioContext: AudioContext | null = null;
	onMount(() => {
		audioContext = new AudioContext();
	});

	// states
	let showRetry = $state({ when: performance.now() - 10000, show: false });
	let showError = $state({ when: performance.now() - 10000, show: false });
	let showClose = $state({ when: performance.now() - 10000, show: false });
	let showConnected = $state({ when: performance.now() - 10000, show: false });
	let roniVisible = $state(false);
	let roniMirror = $state(false);
	let roniBlink = $state(false);
	let roniMouth = $state(false);

	// error icons
	const errorThread = async () => {
		while (true) {
			const now = performance.now();
			showRetry.show = now - showRetry.when < 5000;
			showError.show = now - showError.when < 5000;
			showClose.show = now - showClose.when < 5000;
			showConnected.show = now - showConnected.when < 5000;
			await sleep(200);
		}
	};
	const et = errorThread();

	// blink
	const blinkThread = async () => {
		while (roniVisible) {
			roniBlink = true;
			await sleep(150);
			roniBlink = false;
			await sleep(1000 * random(1, 5));
		}
	};

	// mirror side
	const mirrorThread = async () => {
		while (roniVisible) {
			roniMirror = true;
			await sleep(1000 * random(1, 2));
			roniMirror = false;
			await sleep(1000 * random(5, 8));
		}
	};

	// talking
	const talkingThread = async () => {
		while (roniVisible) {
			roniMouth = !roniMouth;
			await sleep(100 * random(1.5, 2));
		}
	};

	// websocket
	new WebsocketBuilder(PUBLIC_WEBSOCKET_URL)
		.withInstantReconnect(true)
		.withBuffer(new ArrayQueue())
		.withBackoff(new ConstantBackoff(1000))
		.onOpen((_) => {
			showConnected.when = performance.now();
		})
		.onClose((_) => {
			showClose.when = performance.now();
		})
		.onError((_) => {
			showError.when = performance.now();
		})
		.onMessage(async (_, ev) => {
			let bytes = ev.data;
			let msg = await decodeMessage(bytes);
			switch (msg.type) {
				case 'eval':
					roniVisible = true;
					blinkThread();
					mirrorThread();
					break;
				case 'say':
					await playbackAudio(audioContext, msg.req_wav);
					await sleep(1000);
					talkingThread();
					await playbackAudio(audioContext, msg.res_wav);
					await sleep(1000);
					roniVisible = false;
			}
		})
		.onRetry((_) => {
			showRetry.when = performance.now();
		})
		.onReconnect((_) => {
			showConnected.when = performance.now();
		})
		.build();
</script>

<svelte:head>
	<title>Roni AI</title>
	<meta name="description" content="Roni talks" />
</svelte:head>

<section>
	<Roni visible={roniVisible} mirror={roniMirror} blink={roniBlink} mouthOpen={roniMouth}></Roni>

	{#if showRetry.show}
		<enhanced:img
			in:fade={{ duration: 100 }}
			out:fade={{ delay: 3000, duration: 250 }}
			alt="reconnect"
			src={rotate}
			width="48px"
			height="48px"
			style="left: 298px"
		/>
	{/if}
	{#if showError.show}
		<enhanced:img
			in:fade={{ duration: 100 }}
			out:fade={{ delay: 3000, duration: 250 }}
			alt="warn"
			src={triangle}
			width="48px"
			height="48px"
			style="left: 350px"
		/>
	{/if}
	{#if showClose.show}
		<enhanced:img
			in:fade={{ duration: 100 }}
			out:fade={{ delay: 3000, duration: 250 }}
			alt="error"
			src={xmark}
			width="48px"
			height="48px"
			style="left: 402px"
		/>
	{/if}
	{#if showConnected.show}
		<enhanced:img
			in:fade={{ duration: 100 }}
			out:fade={{ delay: 3000, duration: 250 }}
			alt="check"
			src={check}
			width="48px"
			height="48px"
			style="left: 454px"
		/>
	{/if}
</section>

<style>
	img {
		position: absolute;
		top: 552px;
		filter: invert(67%) sepia(25%) saturate(5185%) hue-rotate(291deg) brightness(100%)
			contrast(100%);
	}
</style>
