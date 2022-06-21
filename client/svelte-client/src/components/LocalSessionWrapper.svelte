<script lang="ts">
    import {LocalSession, Session, sessionInputs} from "../store/session";

    export let session: LocalSession;

    let localSessionInputs;

    $: if (session) {
        localSessionInputs = sessionInputs(session);
    }
</script>

{#if session && localSessionInputs}
    <slot inputs={$localSessionInputs}></slot>
{/if}
