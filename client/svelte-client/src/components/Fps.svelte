<script lang="ts">
    import Text from "./Text.svelte";

    export let fps: number;

    let text = null;
    $: text = fps ? `${fps} FPS` : null;
</script>

<Text
        text={text}
        fontSize=12
        fontFamily='Courier New'
        align='left'
        baseline='top'
        x={20}
        y={20}/>

<!-- The following allows this component to nest children -->
<slot></slot>
