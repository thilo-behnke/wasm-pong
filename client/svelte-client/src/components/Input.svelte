<script lang="ts">
    import type {Input} from "../store/session";

    const availableInputs: string[] = ["UP", "DOWN"]
    export let inputs: Input[] = [];

    $: player1Inputs = inputs.filter(({player}) => player === 1).map(({input}) => input)
    $: player2Inputs = inputs.filter(({player}) => player === 2).map(({input}) => input)
</script>

<div class="game_inputs">
    <h4>Player 1</h4>
    {#each availableInputs as input}
        <div class="game_input game_input--{player1Inputs.includes(input) ? 'active' : 'inactive'}">
            {input}
        </div>
    {/each}
    <h4>Player 2</h4>
    {#each availableInputs as input}
        <div class="game_input game_input--{player2Inputs.includes(input) ? 'active' : 'inactive'}">
            {input}
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
