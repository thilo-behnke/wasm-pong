<script lang="ts">
    import LocalSessionWrapper from "./LocalSessionWrapper.svelte";
    import NetworkSessionWrapper from "./NetworkSessionWrapper.svelte";
    import type {Session} from "../store/model/session";
    import {SessionState, SessionType} from "../store/model/session";
    import CopyToClipboard from "./CopyToClipboard.svelte";
    import api from "../api/session";

    export let session: Session;

</script>

<div class="session-wrapper">
    {#if !session}
        <h1>no session</h1>
    {:else if session.type === SessionType.LOCAL}
        <LocalSessionWrapper session={session} let:inputs={inputs} let:objects={objects} let:tick={tick}>
            <slot inputs={inputs} objects={objects} tick={tick}></slot>
        </LocalSessionWrapper>
    {:else}
        <NetworkSessionWrapper session={session} let:inputs={inputs} let:objects={objects} let:tick={tick}>
            <slot inputs={inputs} objects={objects} tick={tick}></slot>
        </NetworkSessionWrapper>
    {/if}
</div>

<style>
    .session-wrapper {
        min-width: 20%;
        display: flex;
        flex-flow: column nowrap;
        align-items: center;
    }
</style>
