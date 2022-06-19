<script lang="ts">
	import { FieldWrapper } from "wasm-app";
	import Canvas from "./Canvas.svelte";
	import Fps from "./Fps.svelte";
	import {keysPressed} from "./game/engine";
	import Input from "./Input.svelte";
	import {setContext} from "svelte";
	import {sessionContext, sessionStore} from "./game/session";
	import Action from "./Action.svelte";

	setContext(sessionContext, sessionStore)

	function handleKeydown({key}) {
		if ($keysPressed.includes(key)) {
			return;
		}
		$keysPressed = [...$keysPressed, key]
	}
	function handleKeyup({key}) {
		if (!$keysPressed.includes(key)) {
			return;
		}
		$keysPressed = $keysPressed.filter(key => key !== key)
	}
</script>

<main>
	{JSON.stringify($sessionStore.session)}
	<Action></Action>
	<div class="game-area">
		<Canvas>
			<Fps></Fps>
		</Canvas>
		<Input inputs={$keysPressed}></Input>
	</div>
</main>
<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup}/>

<style>
	main {
		display: flex;
		flex-flow: column;
		justify-content: start;
		align-items: center;
	}

	.game-area {
		display: flex;
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
