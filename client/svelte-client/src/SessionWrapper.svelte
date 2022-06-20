<script lang="ts">
    import {localSessionInputs, Session, SessionState, SessionType} from "./game/session";
    import LocalSessionWrapper from "./LocalSessionWrapper.svelte";
    import NetworkSessionWrapper from "./NetworkSessionWrapper.svelte";

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
        <h3>waiting for other player...</h3>
    {:else if session.state === SessionState.CLOSED}
        <h3>game over!</h3>
    {:else if session.state === SessionState.RUNNING}
        <NetworkSessionWrapper let:inputs={inputs}>
            <slot inputs={$localSessionInputs}></slot>
        </NetworkSessionWrapper>
    {:else }
        <h3>unknown game state</h3>
    {/if}
{/if}
