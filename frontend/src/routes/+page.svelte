<script lang="ts">
	import Roni from '$lib/components/roni.svelte';

	let mirror = $state(false);
	let blink = $state(false);
	let mouthOpen = $state(false);

	const mirrorClick = () => {
		mirror = !mirror;
	};
	const blinkClick = () => {
		blink = !blink;
	};
	const mouthClick = () => {
		mouthOpen = !mouthOpen;
	};
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	<button onclick={mirrorClick}>Mirror {mirror}</button>
	<button onclick={blinkClick}>Blink {blink}</button>
	<button onclick={mouthClick}>Mouth {mouthOpen}</button>

	<Roni visible={true} {mirror} {blink} {mouthOpen}></Roni>
</section>

<style>
	section {
		background: red;
	}
</style>
