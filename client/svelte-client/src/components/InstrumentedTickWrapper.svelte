<script lang="ts">
    import {gameField} from "../store/engine";
    import {networkTickEvents, sessionInputs} from "../store/session";

    export let killLoopOnError = true;

    let frame: number;

    $: if (networkTickEvents && $networkTickEvents.hasNext) {
        const tick = networkTickEvents.next();
        if (tick != null) {
            gameField.update(tick.objects);
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
