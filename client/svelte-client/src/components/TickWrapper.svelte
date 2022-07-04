<script lang="ts">
    import {onMount} from "svelte";
    import {GameFieldStore} from "../store/engine";

    export let gameFieldStore: GameFieldStore;
    export let inputs;
    export let killLoopOnError = true;

    let frame: number;

    onMount(() => {
        return createLoop((elapsed, dt) => {
            gameFieldStore.tick(inputs, dt);
        });
    })

    function createLoop (fn) {
        let elapsed = 0;
        let lastTime = performance.now();
        (function loop() {
            frame = requestAnimationFrame(loop);
            const beginTime = performance.now();
            const dt = (beginTime - lastTime) / 1000;
            lastTime = beginTime;
            elapsed += dt;
            fn(elapsed, dt);
        })();
        return () => {
            cancelAnimationFrame(frame);
        };
    }

    function handleError(err) {
        console.error(err);
        if (killLoopOnError) {
            cancelAnimationFrame(frame);
            console.warn('Animation loop stopped due to an error');
        }
    }
</script>

<slot tick={$gameFieldStore} inputs={inputs} handleError={handleError}></slot>
