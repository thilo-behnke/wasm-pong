<script lang="ts">
    import {gameField} from "../store/engine";
    import {networkTickEvents, sessionInputs} from "../store/session";
    import type {GameState, HostSessionSnapshot} from "../store/model/session";
    import Input from "./Input.svelte";

    export let killLoopOnError = true;

    let frame: number;

    let lastTick;
    let inputs: Input[];

    $: if (networkTickEvents && $networkTickEvents.hasNext) {
        const tick = networkTickEvents.next() as HostSessionSnapshot;
        if (tick != null) {
            if (lastTick && lastTick.ts >= tick.ts) {
                // TODO: How is this possible?
                console.error(`!!!! Duplicated Tick ${tick.ts} (vs ${lastTick.ts}) !!!!`)
            } else {
                inputs = tick.inputs;
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

<slot tick={$gameField} inputs={inputs} handleError={handleError}></slot>
