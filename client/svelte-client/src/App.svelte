<script lang="ts">
    import {FieldWrapper} from "wasm-app";
    import Canvas from "./Canvas.svelte";
    import Fps from "./Fps.svelte";
    import Input from "./Input.svelte";
    import {setContext} from "svelte";
    import {localSessionInputs, sessionContext, sessionInputs, sessionStore} from "./game/session";
    import ModeSelect from "./ModeSelect.svelte";
    import {network, networkContext} from "./game/network";
    import NetworkSessionWrapper from "./NetworkSessionWrapper.svelte";
    import GameSettings from "./GameSettings.svelte";

    setContext(sessionContext, {session: () => ($sessionStore as any).session, inputs: () => $sessionInputs});
    setContext(networkContext, network);

    let debug = false;

    function localSession() {
        sessionStore.createLocalSession();
    }

    function createSession() {
        $network.loading = true;
        sessionStore.createNetworkSession().then(() => {
            $network.loading = false;
        })
    }

    function joinSession(sessionId) {
        $network.loading = true;
        sessionStore.joinNetworkSession(sessionId).then(() => {
            $network.loading = false;
        })
    }

    function watchSession(sessionId) {
        $network.loading = true;
        sessionStore.watchNetworkSession(sessionId).then(() => {
            $network.loading = false;
        })
    }

    function toggleDebug() {
        debug = !debug;
    }
</script>
<main>
    <h1>Welcome to WASM-Pong!</h1>
    {#if !$sessionStore.session}
        <div class="mode-select">
            <ModeSelect
                    on:local-create={() => localSession()}
                    on:session-create={() => createSession()}
                    on:session-join={({detail: sessionId}) => joinSession(sessionId)}
                    on:session-watch={({detail: sessionId}) => watchSession(sessionId)}
                    on:debug-toggle={() => toggleDebug()}
            ></ModeSelect>
        </div>
    {:else}
        <NetworkSessionWrapper>
            <div class="game-area">
                <Canvas debug={debug}>
                    <Fps></Fps>
                </Canvas>
                <div class="game-area__hud">
                    <GameSettings on:debug-toggle={() => toggleDebug()}></GameSettings>
                    <Input inputs={$sessionInputs}></Input>
                </div>
            </div>
        </NetworkSessionWrapper>
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
