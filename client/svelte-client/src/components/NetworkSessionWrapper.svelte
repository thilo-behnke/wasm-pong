<script lang="ts">

    import {networkEvents, networkSessionStateEvents, sessionInputs} from "../store/session";
    import type {NetworkSession} from "../store/model/session";
    import {SessionState, SessionType} from "../store/model/session";
    import CopyToClipboard from "./CopyToClipboard.svelte";
    import api from "../api/session";
    import type {Readable} from "svelte/store";
    import type {Input} from "../store/model/input";
    import {getPlayerKeyboardInputs} from "../store/input";
    import {gameField} from "../store/engine";

    export let session: NetworkSession;

    let joinLink;
    let cachedSessionId;
    let relevantKeyboardEvents: Readable<Input[]>;

    // TODO: objects must come from events for peer and observer

    $: if(!cachedSessionId && session) {
        cachedSessionId = session.session_id;
        console.log("NetworkSessionWrapper ready, now setting up sessionEvents")
        joinLink = api.createJoinLink(session.session_id);

        relevantKeyboardEvents = getPlayerKeyboardInputs(session.you.nr);
    }

    $: if(session && session.type === SessionType.HOST && session.state === SessionState.RUNNING) {
        console.debug("sending host snapshot")
        networkEvents.produce({inputs: $relevantKeyboardEvents, session_id: session.session_id, objects: $gameField.objects, player_id: session.you.id, ts: $gameField.lastTick})
    }

    $: if(session && session.type === SessionType.PEER && session.state === SessionState.RUNNING) {
        console.debug("sending host snapshot")
        networkEvents.produce({inputs: $relevantKeyboardEvents, session_id: session.session_id, player_id: session.you.id, ts: $gameField.lastTick})
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
        <slot inputs={$sessionInputs} objects={$gameField.objects} tick={gameField.tick}></slot>
    {:else }
        <h3>unknown game state</h3>
    {/if}
{/if}

