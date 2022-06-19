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

    const toggleDebug = () => {
        dispatch("debug-toggle")
    }
</script>

<div class="game-actions">
    <button on:click={() => localSession()}>Create Local Game</button>
    <button on:click={() => createSession()}>Create Online Game</button>
    <input bind:value={sessionId}/>
    <button on:click={() => joinSession()}>Join Online Game</button>
    <button on:click={() => watchSession()}>Watch Online Game</button>
    <button on:click={() => toggleDebug()}>Toggle Debug</button>
</div>

<style>
    .game-actions {
        display: flex;
        justify-content: center;
    }
</style>
