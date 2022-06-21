<script lang="ts">

    import type {NetworkSession} from "../store/session";
    import {sessionInputs} from "../store/session";

    export let session: NetworkSession;
    let networkSessionInputs;

    $: if (session) {
        networkSessionInputs = sessionInputs(session)
    }
</script>

{#if !session}
    <h3>no session</h3>
{:else}
    {session.type.toLowerCase()}
    <slot inputs={$networkSessionInputs}></slot>
{/if}
