<script lang="ts">
    import Text from "./Text.svelte";
    import type {GameState} from "../store/model/session";

    export let dimensions: { width: number, height: number };
    export let state: GameState;
</script>

{#if state?.score}
    <Text
            text={state.score.player_1.toString()}
            fontSize=40
            fontFamily='Courier New'
            align='left'
            baseline='top'
            x={dimensions.width / 2 - 70}
            y={20}/>

    <Text
            text={state.score.player_2.toString()}
            fontSize=40
            fontFamily='Courier New'
            align='left'
            baseline='top'
            x={dimensions.width / 2 + 45}
            y={20}/>
{/if}

<Text
    text={state.winner ? "player " + state.winner + " has won!" : ""}
    fontSize=32
    fontFamily='Courier New'
    align='left'
    baseline='top'
    x={dimensions.width / 4}
    y={dimensions.height / 2}/>
