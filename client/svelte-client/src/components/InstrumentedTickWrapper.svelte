<script lang="ts">
    import {gameField} from "../store/engine";
    import {networkTickEvents, sessionInputs} from "../store/session";
    import type {GameState, HostSessionSnapshot} from "../store/model/session";

    export let killLoopOnError = true;

    let frame: number;

    let lastTick;

    $: if (networkTickEvents && $networkTickEvents.hasNext) {
        const tick = networkTickEvents.next() as HostSessionSnapshot;
        if (tick != null) {
            if (lastTick && lastTick.ts >= tick.ts) {
                // TODO: How is this possible?
                console.warn(`!!!! Duplicated Tick ${tick.ts} (vs ${lastTick.ts}) !!!!`)
            } else {
                gameField.update(tick.objects, tick.state);
                lastTick = tick;
            }
        }
    }

    // TODO: score must come from events for instrumented ticks
    function handleError(err) {
        console.error(err);
        if (killLoopOnError) {
            cancelAnimationFrame(frame);
            console.warn('Animation loop stopped due to an error');
        }
    }
</script>

<slot tick={$gameField} inputs={$sessionInputs} handleError={handleError}></slot>
