<script lang="ts">
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

<enhanced:img alt="Body" src={body} width="400px" height="400px" />
<Head {mirror} {blink} {mouthOpen} />

<style>
	img {
		position: absolute;
		top: 280px;
	}
</style>
