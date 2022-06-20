<script lang="ts">
    import {keyboardInputs, Session, SessionState, SessionType} from "./game/session";
    import LocalSessionWrapper from "./LocalSessionWrapper.svelte";

    export let session: Session;
</script>

{#if !session}
    <h1>no session</h1>
{:else if session.type === SessionType.LOCAL}
    <LocalSessionWrapper let:inputs={inputs}>
        <slot inputs={inputs}></slot>
    </LocalSessionWrapper>
{:else}
    {#if session.state === SessionState.PENDING}
        <h1>waiting for other player...</h1>
    {:else if session.state === SessionState.CLOSED}
        <h1>game over!</h1>
    {:else if session.state === SessionState.RUNNING}
        <slot inputs={$keyboardInputs}></slot>
    {:else }
        <h1>unknown game state</h1>
    {/if}
{/if}
