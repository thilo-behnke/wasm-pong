<script lang="ts">
    import {sessionInputs} from "../store/session";
    import type {LocalSession, Session} from "../store/model/session";

    export let session: LocalSession;

    let localSessionInputs;

    $: if (session) {
        localSessionInputs = sessionInputs(session);
    }
</script>

{#if session && localSessionInputs}
    <slot inputs={$localSessionInputs}></slot>
{/if}
