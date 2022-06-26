<script lang="ts">

    import {networkSessionStateEvents, sessionInputs} from "../store/session";
    import type {NetworkSession} from "../store/model/session";
    import {SessionState} from "../store/model/session";
    import CopyToClipboard from "./CopyToClipboard.svelte";
    import api from "../api/session";

    export let session: NetworkSession;
    let joinLink;

    let cachedSessionId;

    $: if(!cachedSessionId && session) {
        cachedSessionId = session.session_id;
        console.log("NetworkSessionWrapper ready, now setting up sessionEvents")
        joinLink = api.createJoinLink(session.session_id);
    }
</script>

{#if !session}
    <h3>no session</h3>
{:else}
    {JSON.stringify(session)}
    {JSON.stringify($networkSessionStateEvents)}
    {#if session.state === SessionState.PENDING}
        <h3>waiting for other player...</h3>
        <CopyToClipboard text={joinLink}></CopyToClipboard>
    {:else if session.state === SessionState.CLOSED}
        <h3>game over!</h3>
    {:else if session.state === SessionState.RUNNING}
        <slot inputs={$sessionInputs}></slot>
    {:else }
        <h3>unknown game state</h3>
    {/if}
{/if}

