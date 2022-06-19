<script lang="ts">
    const controls: string[] = ["w", "s", "arrowup", "arrowdown"]
    export let inputs: string[] = [];

    $: filteredInputs = inputs.map(i => i.toLowerCase()).filter(i => controls.includes(i))
</script>

<div class="game_inputs">
    {#each controls as control}
        <div class="game_input game_input--{filteredInputs.includes(control) ? 'active' : 'inactive'}">
            {control}
        </div>
    {/each}
</div>


<style>
    .game_inputs {
        display: flex;
        flex-flow: column;
        justify-content: center;
    }

    .game_input {
        background-color: #eee;
        border-radius: 4px;
        font-size: 1em;
        padding: 0.2em 0.5em;
        border-top: 5px solid rgba(255, 255, 255, 0.5);
        border-left: 5px solid rgba(255, 255, 255, 0.5);
        border-right: 5px solid rgba(0, 0, 0, 0.2);
        border-bottom: 5px solid rgba(0, 0, 0, 0.2);
        color: #555;
    }

    .game_input--inactive {
        color: black;
    }

    .game_input--active {
        color: white;
        background-color: #ff3e00;
    }
</style>
