<script lang="ts">
	import { fly } from 'svelte/transition';
	import Head from '$lib/components/head.svelte';
	import body from '$lib/images/body.png';

	let {
		visible,
		mirror,
		blink,
		mouthOpen
	}: {
		visible: boolean;
		mirror: boolean;
		blink: boolean;
		mouthOpen: boolean;
	} = $props();
</script>

{#if visible}
	<div
		in:fly={{ duration: 1000, y: 600, opacity: 1 }}
		out:fly={{ duration: 1000, y: 600, opacity: 1 }}
	>
		<enhanced:img alt="Body" src={body} width="400px" height="400px" />
		<Head {mirror} {blink} {mouthOpen} />
	</div>
{/if}

<style>
	img {
		position: absolute;
		top: 320px;
		left: 200px;
	}
</style>
