<script lang="ts">
    import {getContext} from "svelte";
    import {sessionContext} from "./game/session";

    let session = getContext(sessionContext);
</script>

<div class="game-actions">
    <button on:click={() => session.createSession()}>Create Session</button>
    <button on:click={() => session.joinSession()}>Join Session</button>
    <button on:click={() => session.watchSession()}>Watch Session</button>
</div>

<style>
    .game-actions {
        display: flex;
        justify-content: center;
    }
</style>
