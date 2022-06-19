<script lang="ts">
    import {createEventDispatcher} from "svelte";

    const dispatch = createEventDispatcher();

    let sessionId = '';

    const localSession = () => {
        dispatch("local-create")
    }

    const createSession = () => {
        dispatch("session-create")
    }

    const joinSession = () => {
        if (!sessionId) {
            return
        }
        dispatch("session-join", sessionId)
    }

    const watchSession = () => {
        if (!sessionId) {
            return
        }
        dispatch("session-watch", sessionId)
    }

</script>

<div class="game-mode-select">
    <button on:click={() => localSession()}>Create Local Game</button>
    <button on:click={() => createSession()}>Create Online Game</button>
    <input bind:value={sessionId}/>
    <button disabled={!sessionId} on:click={() => joinSession()}>Join Online Game</button>
    <button disabled={!sessionId} on:click={() => watchSession()}>Watch Online Game</button>
</div>

<style>
    .game-mode-select {
        display: grid;
        grid-auto-columns: 200px;
        grid-auto-rows: 200px;
    }
</style>
