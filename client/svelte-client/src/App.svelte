<script lang="ts">
	import { FieldWrapper } from "wasm-app";
	import Canvas from "./Canvas.svelte";
	import Fps from "./Fps.svelte";
	const field = FieldWrapper.new();
	console.log({field})
	export let name: string;
</script>

<main>
	<Canvas>
		<Fps></Fps>
	</Canvas>
</main>

<style>
	main {
		display: flex;
		justify-content: center;
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
