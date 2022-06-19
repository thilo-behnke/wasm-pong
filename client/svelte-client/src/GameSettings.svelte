<script lang="ts">
    import {createEventDispatcher} from "svelte";

    const dispatch = createEventDispatcher();
    const toggleDebug = () => {
        dispatch("debug-toggle")
    }
</script>

<div class="game-settings">
    <button on:click={() => toggleDebug()}>Toggle Debug</button>
</div>

<style>
    .game-settings {
        display: flex;
        flex-flow: column nowrap;
        align-items: center;
    }
</style>
