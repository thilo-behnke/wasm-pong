<script lang="ts">
    import {createEventDispatcher} from "svelte";

    const dispatch = createEventDispatcher();

    const createSession = () => {
        dispatch("session-create")
    }

    const joinSession = (sessionId) => {
        dispatch("session-join", sessionId)
    }

    const watchSession = (sessionId) => {
        dispatch("session-watch", sessionId)
    }
</script>

<div class="game-actions">
    <button on:click={() => createSession()}>Create Session</button>
    <button on:click={() => joinSession("session")}>Join Session</button>
    <button on:click={() => watchSession("session")}>Watch Session</button>
</div>

<style>
    .game-actions {
        display: flex;
        justify-content: center;
    }
</style>
