<script lang="ts">

    import {networkEvents, networkSessionStateEvents, sessionInputs} from "../store/session";
    import type {NetworkSession} from "../store/model/session";
    import {SessionState, SessionType} from "../store/model/session";
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
    $: if(session) {
        switch(session.type) {
            case SessionType.HOST:
                networkEvents.produce({inputs: $sessionInputs, session_id: session.session_id, objects: [], player: session.you.id, ts: Date.now()})
                break;
            case SessionType.PEER:
                networkEvents.produce({inputs: $sessionInputs, session_id: session.session_id, player: session.you.id, ts: Date.now()})
                break;
            case SessionType.OBSERVER:
                networkEvents.produce({session_id: session.session_id, player: session.you.id, ts: Date.now()})
                break;
            default:
                throw new Error("session snapshot update not implemented for session type " + session.type);
        }
    }
</script>

{#if !session}
    <h3>no session</h3>
{:else}
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

