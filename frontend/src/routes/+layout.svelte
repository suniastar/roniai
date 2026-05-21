<script lang="ts">
	let { children } = $props();
</script>

<div class="app">
	<main>
		{@render children()}
	</main>
</div>
