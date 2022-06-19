<script lang="ts">
    import {FieldWrapper} from "wasm-app";
    import Canvas from "./Canvas.svelte";
    import Fps from "./Fps.svelte";
    import {keysPressed} from "./game/engine";
    import Input from "./Input.svelte";
    import {setContext} from "svelte";
    import {sessionContext, sessionStore} from "./game/session";
    import Action from "./Action.svelte";
    import {network, networkContext} from "./game/network";

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
        debug = true;
    }
</script>
<main>
    {#if $network.loading}
        loading...
    {:else}
        {JSON.stringify($sessionStore)}
    {/if}
    {#if !$sessionStore.session}
        <div class="mode-select">
            <Action
                    on:local-create={() => localSession()}
                    on:session-create={() => createSession()}
                    on:session-join={({detail: sessionId}) => joinSession(sessionId)}
                    on:session-watch={({detail: sessionId}) => watchSession(sessionId)}
                    on:debug-toggle={() => toggleDebug()}
            ></Action>
        </div>
    {:else}
        <div class="game-area">
            <Canvas debug={debug}>
                <Fps></Fps>
            </Canvas>
            <Input inputs={$keysPressed}></Input>
        </div>
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

    }

    .game-area {
        display: flex;
    }

    h1 {
        color: #ff3e00;
        text-transform: uppercase;
        font-size: 4em;
        font-weight: 100;
    }

    @media (min-width: 640px) {
        main {
            max-width: none;
        }
    }
</style>
