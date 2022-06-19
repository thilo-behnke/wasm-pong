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
        border: 1px solid grey;
        border-radius: 0.5em;
        text-align: center;
    }

    .game_input--inactive {
        color: black;
    }

    .game_input--active {
        color: white;
        background-color: red;
    }
</style>
