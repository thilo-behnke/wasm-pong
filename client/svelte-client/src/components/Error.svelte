<script lang="ts">
    export let error: string;
    export let duration: number;
</script>

{#if error}
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
