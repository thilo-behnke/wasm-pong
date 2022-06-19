<script lang="ts">
    import {FieldWrapper} from "wasm-app";
    import Canvas from "./Canvas.svelte";
    import Fps from "./Fps.svelte";
    import {keysPressed} from "./game/engine";
    import Input from "./Input.svelte";
    import {setContext} from "svelte";
    import {sessionContext, sessionStore} from "./game/session";
    import ModeSelect from "./ModeSelect.svelte";
    import {network, networkContext} from "./game/network";
    import NetworkSessionWrapper from "./NetworkSessionWrapper.svelte";
    import GameSettings from "./GameSettings.svelte";

    setContext(sessionContext, sessionStore);
    setContext(networkContext, network);

    let debug = false;

    function handleKeydown({key}) {
        if ($keysPressed.includes(key)) {
            return;
        }
        $keysPressed = [...$keysPressed, key]
    }

    function handleKeyup({key}) {
        if (!$keysPressed.includes(key)) {
            return;
        }
        $keysPressed = $keysPressed.filter(key => key !== key)
    }

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
    {#if !$sessionStore.session}
        <div class="mode-select">
            <h1>Welcome to WASM-Pong!</h1>
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
                <div>
                    <GameSettings on:debug-toggle={() => toggleDebug()}></GameSettings>
                    <Input inputs={$keysPressed}></Input>
                </div>
            </div>
        </NetworkSessionWrapper>
    {/if}
</main>
<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup}/>

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
        display: flex;
    }

    @media (min-width: 640px) {
        main {
            max-width: none;
        }
    }
</style>
