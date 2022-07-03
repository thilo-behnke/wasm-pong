<script lang="ts">
    import {renderable} from "../store/engine";
    import {drawLine, mainColor} from "../store/render";

    export let x = 0;
    export let y = 0;
    export let height = 0;
    export let dashed: boolean = false;

    renderable(props => {
        const { engineCtx: ctx } = props;
        ctx.fillStyle = mainColor;
        if (dashed) {
            ctx.setLineDash([5, 15]);
        }
        drawLine(ctx, x, y, x, height, mainColor);
        ctx.setLineDash([]);
    });
</script>

<slot></slot>
