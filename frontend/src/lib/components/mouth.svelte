<script lang="ts">
	import mouth_open from '$lib/images/mouth_open.png';
	import mouth_close from '$lib/images/mouth_close.png';

	let {
		mirror,
		open
	}: {
		mirror: boolean;
		open: boolean;
	} = $props();
</script>

<enhanced:img
	alt="Mouth"
	src={mouth_open}
	width="400px"
	height="400px"
	class={[open && 'hide', mirror && 'mirror']}
/>
<enhanced:img
	alt="Mouth"
	src={mouth_close}
	width="400px"
	height="400px"
	class={[!open && 'hide', mirror && 'mirror']}
/>

<style>
	img {
		position: absolute;
	}

	.hide {
		display: none;
	}

	.mirror {
		-webkit-transform: scale(-1, 1);
		-moz-transform: scale(-1, 1);
		-o-transform: scale(-1, 1);
		transform: scale(-1, 1);
	}
</style>
