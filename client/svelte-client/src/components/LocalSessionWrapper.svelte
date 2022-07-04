<script lang="ts">
    import {sessionInputs} from "../store/session";
    import type {LocalSession} from "../store/model/session";
    import TickWrapper from "./TickWrapper.svelte";
    import {gameField} from "../store/engine";

    export let session: LocalSession;
</script>

{#if session && sessionInputs}
    <TickWrapper gameFieldStore={gameField} inputs={$sessionInputs} let:tick={tick} let:inputs={inputs} let:handleError={handleError}>
        <slot inputs={inputs} tick={tick} handleError={handleError} events={[]}></slot>
    </TickWrapper>
{/if}
