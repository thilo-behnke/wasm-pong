<script lang="ts">

    import type {GameScore} from "../store/engine";
    import Text from "./Text.svelte";

    export let dimensions: { width: number, height: number };
    export let score: GameScore;
</script>

{#if score}
    <Text
            text={score.player1.toString()}
            fontSize=40
            fontFamily='Courier New'
            align='left'
            baseline='top'
            x={dimensions.width / 2 - 70}
            y={20}/>

    <Text
            text={score.player2.toString()}
            fontSize=40
            fontFamily='Courier New'
            align='left'
            baseline='top'
            x={dimensions.width / 2 + 45}
            y={20}/>
{/if}
