<script lang="ts">

    import {sessionInputs} from "../store/session";
    import type {NetworkSession} from "../store/model/session";

    export let session: NetworkSession;
    let networkSessionInputs;

    $: if (session) {
        networkSessionInputs = sessionInputs(session)
    }
</script>

{#if !session}
    <h3>no session</h3>
{:else}
    <slot inputs={$networkSessionInputs}></slot>
{/if}
