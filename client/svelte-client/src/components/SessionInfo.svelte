<script lang="ts">
    import type {Session} from "../store/model/session";
    import {isLocalSession, isNetworkSession, isPlayer, SessionType} from "../store/model/session";

    export let session: Session;
</script>

{#if session}
    <div class="session-info">
        <span><b>Id:</b> {session.session_id}</span>
        <span><b>State:</b> {session.state}</span>
        {#if isNetworkSession(session)}
            <span><b>Type:</b> {session.type}</span>
            {#if isPlayer(session.you)}
                <span><b>You:</b> Player {session.you.nr} ({session.you.id})</span>
            {:else}
                <span><b>You:</b> Observer ({session.you.id})</span>
            {/if}
        {/if}
    </div>
{/if}

<style>
    .session-info {
        display: flex;
        flex-flow: row wrap;
        font-size: 0.9rem;
    }

    .session-info > span + span {
        margin-left: 1rem;
    }
</style>
