<script lang="ts">
    import {gameField} from "../store/engine";
    import {gameStateEvents, networkTickEvents, sessionInputs} from "../store/session";
    import type {GameState} from "../store/model/session";

    export let killLoopOnError = true;

    let frame: number;

    let state: GameState;
    $: state = $gameStateEvents;

    $: if (networkTickEvents && $networkTickEvents.hasNext) {
        const tick = networkTickEvents.next();
        if (tick != null) {
            gameField.update(tick.objects, state);
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
