<script lang="ts">
    import {getContext} from "svelte";
    import {sessionContext, SessionState} from "./game/session";
    import Canvas from "./Canvas.svelte";

    export let debug;

    let session = getContext(sessionContext);

</script>

{#if !$session.session}
    error!
{:else if $session.session.state === SessionState.PENDING}
    waiting for other player...
{:else if $session.session.state === SessionState.CLOSED}
    game over!
{:else if $session.session.state === SessionState.RUNNING}
    <slot></slot>
{:else }
    unknown game state
{/if}


