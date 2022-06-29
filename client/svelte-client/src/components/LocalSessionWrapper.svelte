<script lang="ts">
    import {sessionInputs} from "../store/session";
    import type {LocalSession} from "../store/model/session";
    import {gameField} from "../store/engine";

    export let session: LocalSession;

    const tick = (dt: number) => {
        gameField.tick($sessionInputs, dt)
    }
</script>

{#if session && sessionInputs}
    <slot inputs={$sessionInputs} objects={$gameField.objects} tick={tick}></slot>
{/if}
