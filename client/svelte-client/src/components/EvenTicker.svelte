<script lang="ts">
    import type {GameEventWrapper} from "../store/model/event";

    export let events: GameEventWrapper[];

    const eventsCached = 1_000;

    let lastEvents = [];
    $: if(events) {
        lastEvents = [...lastEvents, ...events].slice(lastEvents.length + events.length - eventsCached);
    }
</script>

<div class="event-ticker">
    {#if events.length }
        {#each lastEvents as event }
            <div>{JSON.stringify(event)}</div>
        {/each}
    {:else}
        No events available.
    {/if}
</div>


<style>
    .event-ticker {
        width: 100%;

        overflow-y: scroll;
        font-size: 0.8rem;
    }
</style>
