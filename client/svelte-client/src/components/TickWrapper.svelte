<script lang="ts">
    import {onMount} from "svelte";
    import {GameFieldStore} from "../store/engine";

    export let gameFieldStore: GameFieldStore;
    export let inputs;
    export let killLoopOnError = true;

    const targetFps = 60;
    const frameThreshold = 1_000 / targetFps;

    let frame: number;

    onMount(() => {
        return createLoop((elapsed, dt) => {
            gameFieldStore.tick(inputs, dt);
        });
    })

    function createLoop (fn) {
        let elapsed = 0;
        let lastTime = Date.now();
        (function loop() {
            frame = requestAnimationFrame(loop);
            const now = Date.now();
            const dtMs = now - lastTime;
            if (dtMs < frameThreshold) {
                return;
            }
            lastTime = now - (dtMs % frameThreshold);
            elapsed += frameThreshold;
            fn(elapsed / 1_000, frameThreshold / 1_000);
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
