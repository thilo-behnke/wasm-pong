<script lang="ts">
    import {createEventDispatcher, getContext, onMount} from "svelte";
    import {Shadow} from 'svelte-loading-spinners'
    import session from "../api/session";

    export let isLoading = false;

    const dispatch = createEventDispatcher();

    let joinSessionId = '';
    let watchSessionId = '';

    $: disableControls = isLoading;

    onMount(() => {
        if (!window.location.search.startsWith("?")) {
            return;
        }
        const params = window.location.search.slice(1).split("&").map(p => p.split('=')).reduce((acc, [key, val]) => ({...acc, [key]: val}), {}) as any;
        if (params.join) {
            joinSessionId = params.join;
        }
        if (params.watch) {
            watchSessionId = params.watch;
        }
    })

    const localSession = () => {
        dispatch("local-create")
    }

    const createSession = () => {
        dispatch("session-create")
    }

    const joinSession = () => {
        if (!joinSessionId) {
            return
        }
        dispatch("session-join", joinSessionId)
    }

    const watchSession = () => {
        if (!watchSessionId) {
            return
        }
        dispatch("session-watch", watchSessionId)
    }

</script>

<div class="game-mode-select">
    {#if isLoading}
        <h3 style="text-align: center">Loading...</h3>
        <div class="game-mode-select__loading">
            <Shadow size="20" unit="px" color="#FF3E00" duration="1s"></Shadow>
        </div>
    {:else}
        <h3 style="text-align: center">Please select a game mode</h3>
    {/if}
    <button disabled={disableControls} on:click={() => localSession()}>Create Local Game</button>
    <hr/>
    <button disabled={disableControls} on:click={() => createSession()}>Create Online Game</button>
    <div class="game-mode-select__group">
        <input bind:value={joinSessionId} placeholder="session id"/>
        <button disabled={!joinSessionId || disableControls} on:click={() => joinSession()}>Join Online Game</button>
    </div>
    <div class="game-mode-select__group ">
        <input bind:value={watchSessionId} placeholder="session id"/>
        <button disabled={!watchSessionId || disableControls} on:click={() => watchSession()}>Watch Online Game</button>
    </div>
</div>

<style>
    .game-mode-select {
        display: grid;
        min-width: 20%;
    }

    .game-mode-select > hr {
        width: 100%;
        margin-bottom: 20px;
    }

    .game-mode-select__loading {
        display: flex;
        justify-content: center;
        padding: 20px;
    }

    .game-mode-select__group {
        display: grid;
        grid-template-columns: 1fr 200px;
        grid-column-gap: 10px;
    }

    .game-mode-select__group > input {
    }
</style>
