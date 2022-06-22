<script lang="ts">
    export let text: string;

    let copied = false;

    const copyToClipboard = () => {
        navigator.clipboard.writeText(text);
        copied = true;
    }
</script>

{#if text}
    <div class="copy-to-clipboard">
        <input type="text" bind:value={text} disabled/>
        <button on:click={() => copyToClipboard()}>Copy{#if copied} <span class="copied-icon">&#10003;</span>{/if}</button>
    </div>
{/if}

<style>
    .copy-to-clipboard {
        display: flex;
        flex-flow: row nowrap;
        justify-content: center;
        width: 100%;
    }

    .copy-to-clipboard > input {
        flex-grow: 1;
    }

    .copy-to-clipboard > button {
        margin-left: 0.4rem;

        display: flex;
        align-items: center;
    }

    .copied-icon {
        font-size: 1.5rem;
        color: #ff3e00;
    }

</style>
