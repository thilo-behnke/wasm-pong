<script lang="ts">
    import {sessionInputs} from "../store/session";
    import type {LocalSession} from "../store/model/session";
    import {gameField} from "../store/engine";

    export let session: LocalSession;
</script>

{#if session && sessionInputs}
    <slot inputs={$sessionInputs} objects={$gameField} tick={gameField.tick}></slot>
{/if}
