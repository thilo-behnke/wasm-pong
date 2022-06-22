<script lang="ts">
    import LocalSessionWrapper from "./LocalSessionWrapper.svelte";
    import NetworkSessionWrapper from "./NetworkSessionWrapper.svelte";
    import type {Session} from "../store/model/session";
    import {SessionState, SessionType} from "../store/model/session";
    import CopyToClipboard from "./CopyToClipboard.svelte";
    import api from "../api/session";

    export let session: Session;

    let joinLink;
    $: if(session) {
        console.log(session);
        joinLink = api.createJoinLink(session.session_id);
    }
</script>

<div class="session-wrapper">
    {#if !session}
        <h1>no session</h1>
    {:else if session.type === SessionType.LOCAL}
        <LocalSessionWrapper session={session} let:inputs={inputs}>
            <slot inputs={inputs}></slot>
        </LocalSessionWrapper>
    {:else}
        {#if session.state === SessionState.PENDING}
            <h3>waiting for other player...</h3>
            <CopyToClipboard text={joinLink}></CopyToClipboard>
        {:else if session.state === SessionState.CLOSED}
            <h3>game over!</h3>
        {:else if session.state === SessionState.RUNNING}
            <NetworkSessionWrapper let:inputs={inputs}>
                <slot inputs={inputs}></slot>
            </NetworkSessionWrapper>
        {:else }
            <h3>unknown game state</h3>
        {/if}
    {/if}
</div>

<style>
    .session-wrapper {
        min-width: 20%;
        display: flex;
        flex-flow: column nowrap;
        align-items: center;
    }
</style>
