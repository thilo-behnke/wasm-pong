<script lang="ts">
    import {sessionInputs} from "../store/session";
    import type {LocalSession, Session} from "../store/model/session";

    export let session: LocalSession;
</script>

{#if session && sessionInputs}
    <slot inputs={$sessionInputs}></slot>
{/if}
