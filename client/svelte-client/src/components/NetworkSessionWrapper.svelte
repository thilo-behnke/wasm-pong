<script lang="ts">

    import type {NetworkSession} from "../store/session";
    import {
        hostNetworkSessionInputs,
        observerNetworkSessionInputs,
        peerNetworkSessionInputs
    } from "../store/session";

    export let session: NetworkSession;
</script>

{#if !session}
    <h3>no session</h3>
{:else if session.type === 'HOST'}
    <h3>Host</h3>
    <slot inputs={$hostNetworkSessionInputs}></slot>
{:else if session.type === 'PEER'}
    <h3>Peer</h3>
    <slot inputs={$peerNetworkSessionInputs}></slot>
{:else if session.type === 'OBSERVER'}
    <h3>Peer</h3>
    <slot inputs={$observerNetworkSessionInputs}></slot>
{/if}
