<script lang="ts">
	import { FieldWrapper } from "wasm-app";
	import Canvas from "./Canvas.svelte";
	const field = FieldWrapper.new();
	console.log({field})
	export let name: string;
</script>

<main>
	<Canvas></Canvas>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>
