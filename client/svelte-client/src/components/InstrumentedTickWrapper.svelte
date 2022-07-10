<script lang="ts">
    import {gameField} from "../store/engine";
    import {gameStateEvents, networkTickEvents, sessionInputs} from "../store/session";
    import type {GameState} from "../store/model/session";

    export let killLoopOnError = true;

    let frame: number;

    let state: GameState;
    $: state = $gameStateEvents;

    let lastTick;

    $: if (networkTickEvents && $networkTickEvents.hasNext) {
        const tick = networkTickEvents.next();
        if (tick != null) {
            console.warn(`Received tick: ${tick.tick}`)
            if (lastTick && lastTick.tick >= tick.tick) {
                console.error(`???? DUPLICATED TICK: ${JSON.stringify(tick)} (vs ${lastTick.tick}) ????`)
            } else {
                console.warn(`!!!! Valid tick: ${JSON.stringify(tick)} !!!!`)
                gameField.update(tick.objects, state);
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
