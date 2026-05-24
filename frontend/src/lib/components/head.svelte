<script lang="ts">
	import Mouth from '$lib/components/mouth.svelte';
	import head from '$lib/images/head.png';
	import eyes from '$lib/images/blink.png';

	let {
		mirror,
		blink,
		mouthOpen
	}: {
		mirror: boolean;
		blink: boolean;
		mouthOpen: boolean;
	} = $props();
</script>

<img alt="Head" src={head} width="400px" height="400px" class={[mirror && 'mirror']} />
{#if blink}
	<img alt="Blink" src={eyes} width="400px" height="400px" class={[mirror && 'mirror']} />
{/if}
<Mouth {mirror} open={mouthOpen} />

<style>
	img {
		position: absolute;
		top: 20px;
		left: 200px;
	}

	.mirror {
		-webkit-transform: scale(-1, 1);
		-moz-transform: scale(-1, 1);
		-o-transform: scale(-1, 1);
		transform: scale(-1, 1);
	}
</style>
