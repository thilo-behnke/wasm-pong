<script>
    import {renderable} from "./game/engine";

    export let color = 'hsl(0, 0%, 100%)';
    export let align = 'center';
    export let baseline = 'middle';

    export let text = '';
    export let x = 0;
    export let y = 0;

    export let fontSize = 16;
    export let fontFamily = '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica';

    renderable(props => {
        const { engineCtx: ctx } = props;
        if (text) {
            ctx.fillStyle = 'black';
            ctx.font = `${fontSize}px ${fontFamily}`;
            ctx.textAlign = align;
            ctx.textBaseline = baseline;
            ctx.fillText(text, x, y);
        }
    });
</script>

<!-- The following allows this component to nest children -->
<slot></slot>
