<script lang="ts">
    import Canvas from "./components/Canvas.svelte";
    import Fps from "./components/Fps.svelte";
    import Input from "./components/Input.svelte";
    import {localSession, networkSession, SessionStore} from "./store/session";
    import ModeSelect from "./components/ModeSelect.svelte";
    import GameSettings from "./components/GameSettings.svelte";
    import SessionWrapper from "./components/SessionWrapper.svelte";
    import Error from "./components/Error.svelte";
    import SessionInfo from "./components/SessionInfo.svelte";
    import type {Readable} from "svelte/store";
    import {SessionType} from "./store/model/session";
    import EvenTicker from "./components/EvenTicker.svelte";
    import Line from "./components/Line.svelte";
    import Score from "./components/Score.svelte";

    let sessionStore: Readable<SessionStore>;
    let debug = false;

    $: loading = $sessionStore?.loading;
    $: error = $sessionStore?.error;
    $: session = $sessionStore?.session;

    function createLocalSession() {
        sessionStore = localSession();
    }

    function createSession() {
        sessionStore = networkSession(SessionType.HOST)
    }

    function joinSession(sessionId) {
        sessionStore = networkSession(SessionType.PEER, sessionId);
    }

    function watchSession(sessionId) {
        sessionStore = networkSession(SessionType.OBSERVER, sessionId);
    }

    function toggleDebug() {
        debug = !debug;
    }
</script>

<main>
    <h1>Welcome to WASM-Pong!</h1>
    {#key error?.at}
        <Error error={error?.value} duration={5_000}></Error>
    {/key}
    {#if !session}
        <div class="mode-select">
            <ModeSelect
                    isLoading={loading}
                    on:local-create={() => createLocalSession()}
                    on:session-create={() => createSession()}
                    on:session-join={({detail: sessionId}) => joinSession(sessionId)}
                    on:session-watch={({detail: sessionId}) => watchSession(sessionId)}
                    on:debug-toggle={() => toggleDebug()}
            ></ModeSelect>
        </div>
    {:else}
        <SessionWrapper session={session} let:inputs={inputs} let:tick={tick} let:events={events} let:handleError={handleError}>
            <div class="game-area">
                <div class="game-area__session">
                    <SessionInfo session={session}></SessionInfo>
                </div>
                <div class="game-area__canvas">
                    <Canvas debug={debug} session={session} inputs={inputs} tick={tick} handleError={handleError} let:dimensions={dimensions}>
                        {#if debug}
                            <Fps fps={tick.meta.fps}></Fps>
                        {/if}
                        <Line x={dimensions.width / 2} y={0} height={dimensions.height} dashed={true}></Line>
                        <Score dimensions={dimensions} state={tick?.state}></Score>
                    </Canvas>
                </div>
                <div class="game-area__hud">
                    <GameSettings on:debug-toggle={() => toggleDebug()}></GameSettings>
                    <Input inputs={inputs}></Input>
                </div>
                <div class="game-area__events">
                    <EvenTicker events={events}></EvenTicker>
                </div>
            </div>
        </SessionWrapper>
    {/if}
</main>

<style>
    main {
        display: flex;
        flex-flow: column;
        justify-content: start;
        align-items: center;
        width: 600px;
        margin: auto;
    }

    .mode-select {
        display: flex;
        flex-flow: column nowrap;
        align-items: center;
        width: 100%;
    }

    .game-area {
        display: grid;
        grid-template-areas:
            "session session"
            "game hud"
            "events events";
        grid-template-rows: min-content 1fr;
        grid-template-columns: 1fr min-content;
        grid-row-gap: 1rem;
        grid-column-gap: 1rem;
    }

    .game-area__session {
        grid-area: session;
        display: flex;
        justify-content: center;
        border: 1px solid #ff3e00;
        padding: 0.6rem;
    }

    .game-area__canvas {
        grid-area: game;
    }

    .game-area__hud {
        grid-area: hud;
        display: grid;
        grid-template-rows: max-content 1fr;
        border: 1px solid #ff3e00;
        padding: 0.4rem;
    }

    .game-area__events {
        grid-area: events;
        display: grid;
        border: 1px solid #ff3e00;
        padding: 0.4rem;
        height: 300px;
    }

    @media (min-width: 640px) {
        main {
            max-width: none;
        }
    }
</style>
