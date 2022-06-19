<script lang="ts">
    import {getContext} from "svelte";
    import {sessionContext, SessionState} from "./game/session";
    import Canvas from "./Canvas.svelte";

    export let debug;

    let session = getContext(sessionContext);

</script>

{#if !$session.session}
    <h1>error!</h1>
{:else if $session.session.state === SessionState.PENDING}
    <h1>waiting for other player...</h1>
{:else if $session.session.state === SessionState.CLOSED}
    <h1>game over!</h1>
{:else if $session.session.state === SessionState.RUNNING}
    <slot></slot>
{:else }
    <h1>unknown game state</h1>
{/if}


