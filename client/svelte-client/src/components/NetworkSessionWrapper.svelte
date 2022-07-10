<script lang="ts">

    import {networkEvents, networkTickEvents, networkSessionStateEvents, sessionInputs} from "../store/session";
    import type {GameState, NetworkSession} from "../store/model/session";
    import {isObserver, isPlayer, SessionState, SessionType} from "../store/model/session";
    import CopyToClipboard from "./CopyToClipboard.svelte";
    import api from "../api/session";
    import type {Readable} from "svelte/store";
    import type {Input} from "../store/model/input";
    import {getPlayerKeyboardInputs} from "../store/input";
    import {gameField} from "../store/engine";
    import InstrumentedTickWrapper from "./InstrumentedTickWrapper.svelte";
    import TickWrapper from "./TickWrapper.svelte";

    export let session: NetworkSession;

    let joinLink;
    let watchLink;
    let cachedSessionId;
    let relevantKeyboardEvents: Readable<Input[]>;

    let lastTick: number = null;

    $: if (!cachedSessionId && session && isPlayer(session.you)) {
        cachedSessionId = session.session_id;
        console.log("NetworkSessionWrapper ready, now setting up sessionEvents")
        joinLink = api.createJoinLink(session.session_id);
        watchLink = api.createWatchLink(session.session_id);

        relevantKeyboardEvents = getPlayerKeyboardInputs(session.you.nr);
    }

    $: if (session && session.type === SessionType.HOST && session.state === SessionState.RUNNING) {
        if (lastTick != $gameField.ts) {
            console.debug("sending host snapshot")
            const state: GameState = $gameField.state;
            networkEvents.produce({
                state,
                inputs: $sessionInputs,
                session_id: session.session_id,
                objects: $gameField.objects,
                player_id: session.you.id,
                ts: $gameField.ts
            });
            lastTick = $gameField.ts;
        }
    }

    $: if (session && session.type === SessionType.PEER && session.state === SessionState.RUNNING) {
        if (lastTick != $gameField.ts) {
            console.debug("sending peer snapshot")
            networkEvents.produce({
                inputs: $relevantKeyboardEvents,
                session_id: session.session_id,
                player_id: session.you.id,
                ts: $gameField.ts
            })
            lastTick = $gameField.ts;
        }
    }

    $: console.debug($networkSessionStateEvents)
</script>

{#if !session}
    <h3>no session</h3>
{:else}
    {#if session.state === SessionState.PENDING}
        <h3>waiting for other player...</h3>
        <CopyToClipboard text={joinLink}></CopyToClipboard>
    {:else if session.state === SessionState.CLOSED}
        <h3>game over!</h3>
    {:else if session.state === SessionState.RUNNING}
        <CopyToClipboard text={watchLink}></CopyToClipboard>
        {#if session.type === SessionType.HOST}
            <TickWrapper gameFieldStore={gameField} inputs={$sessionInputs} throttle={true} let:tick={tick} let:inputs={inputs} let:handleError={handleError}>
                <slot inputs={inputs} tick={tick} events={$networkSessionStateEvents}></slot>
            </TickWrapper>
        {:else}
            <InstrumentedTickWrapper inputs={$sessionInputs} let:tick={tick} let:inputs={inputs}>
                <slot inputs={inputs} tick={tick} events={$networkSessionStateEvents}></slot>
            </InstrumentedTickWrapper>
        {/if}
    {:else }
        <h3>unknown game state</h3>
    {/if}
{/if}

