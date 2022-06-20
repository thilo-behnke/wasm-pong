<script lang="ts">
    import Canvas from "./Canvas.svelte";
    import Fps from "./Fps.svelte";
    import Input from "./Input.svelte";
    import {sessionStore} from "./game/session";
    import ModeSelect from "./ModeSelect.svelte";
    import {network} from "./game/network";
    import GameSettings from "./GameSettings.svelte";
    import SessionWrapper from "./SessionWrapper.svelte";
    import api from "./game/api";

    let debug = false;

    function localSession() {
        sessionCreator(() => api.createLocalSession());
    }

    function createSession() {
        sessionCreator(() => api.createNetworkSession());
    }

    function joinSession(sessionId) {
        sessionCreator(() => api.joinNetworkSession(sessionId));
    }

    function watchSession(sessionId) {
        sessionCreator(() => api.watchNetworkSession(sessionId));
    }

    function sessionCreator(fn) {
        $network.loading = true;
        fn().then(s => {
            $sessionStore = s;
            $network.loading = false;
        })
    }

    function toggleDebug() {
        debug = !debug;
    }
</script>
<main>
    <h1>Welcome to WASM-Pong!</h1>
    {#if !$sessionStore}
        <div class="mode-select">
            <ModeSelect
                    isLoading={$network.loading}
                    on:local-create={() => localSession()}
                    on:session-create={() => createSession()}
                    on:session-join={({detail: sessionId}) => joinSession(sessionId)}
                    on:session-watch={({detail: sessionId}) => watchSession(sessionId)}
                    on:debug-toggle={() => toggleDebug()}
            ></ModeSelect>
        </div>
    {:else}
        <SessionWrapper session={$sessionStore} let:inputs={inputs}>
            <div class="game-area">
                <Canvas debug={debug} session={$sessionStore} inputs={inputs}>
                    <Fps></Fps>
                </Canvas>
                <div class="game-area__hud">
                    <GameSettings on:debug-toggle={() => toggleDebug()}></GameSettings>
                    <Input inputs={inputs}></Input>
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
    }

    .mode-select {
        display: flex;
        flex-flow: column nowrap;
        align-items: center;
        width: 100%;
    }

    .game-area {
        display: grid;
        grid-template-columns: 1fr min-content;
        grid-column-gap: 1rem;
    }

    .game-area__hud {
        display: grid;
        grid-template-rows: max-content 1fr;
        border: 1px solid lightgrey;
        padding: 0.4rem;
    }

    @media (min-width: 640px) {
        main {
            max-width: none;
        }
    }
</style>
