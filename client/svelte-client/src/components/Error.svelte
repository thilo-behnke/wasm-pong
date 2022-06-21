<script lang="ts">
    import {timer} from "../store/utils";

    export let error: string;
    export let duration: number;

    let durationTimer;
    $: if (error) {
        durationTimer = timer(duration || 2_000)
    }
</script>

{#if error && $durationTimer > 0}
    <div class="error">
        {error}
    </div>
{/if}

<style>
    .error {
        border: 0.1rem solid #ff3e00;
        padding: 1rem;
    }
</style>
