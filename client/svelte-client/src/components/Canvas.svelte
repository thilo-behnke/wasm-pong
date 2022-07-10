<script lang="ts">
    import {createEventDispatcher, onMount, setContext} from "svelte";
    import {drawObjects} from "../store/render";
    import {
        engineCanvas,
        engineCtx,
        GameFieldState,
        height,
        pixelRatio,
        props,
        renderContext,
        width
    } from "../store/engine";
    import type {Input} from "../store/model/input";
    import type {GameObject, Session} from "../store/model/session";
    import {SessionType} from "../store/model/session";

    export let inputs: Input[] = []
    export let tick: GameFieldState = null;
    export let session: Session;
    export let handleError: (err: string) => void;

    export let debug = false;

    let canvas: any;
    let ctx: any;
    let listeners = [];

    let renderOnly = false;

    $: if(session) {
        renderOnly = session.type === SessionType.PEER || session.type === SessionType.OBSERVER;
    }

    $: if(canvas && session && tick) {
        render(tick)
    }

    onMount(() => {
        ctx = canvas.getContext('2d');
        engineCtx.set(ctx)
        engineCanvas.set(engineCanvas);
        width.set(canvas.width);
        height.set(canvas.height);
        // field.set_dimensions(canvas.width, canvas.height);
    })

    setContext(renderContext, {
        async add (entity) {
            this.remove(entity);
            listeners.push(entity);

            if (entity.ready) {
                return;
            }

            if (entity.setup) {
                let p = entity.setup($props);
                if (p && p.then) await p;
            }
            entity.ready = true;
        },
        remove (fn) {
            const idx = listeners.indexOf(fn);
            if (idx >= 0) listeners.splice(idx, 1);
        }
    });

    function render({objects, ts}: {objects, ts}) {
        const [canvas_width, canvas_height] = [canvas.width, canvas.height];
        ctx.clearRect(0, 0, canvas_width, canvas_height);
        drawObjects(ctx, objects, [canvas_width, canvas_height], debug);

        listeners.forEach(entity => {
            try {
                if (entity.mounted && entity.ready && entity.render) {
                    entity.render($props, ts);
                }
            } catch (err) {
                handleError(err);
            }
        });
    }

    function handleResize () {
        // TODO: Resolution scaling needs to be implemented in wasm module.
        // width.set(window.innerWidth);
        // height.set(window.innerHeight);
        pixelRatio.set(window.devicePixelRatio);
    }
</script>
<canvas
        bind:this={canvas}
        width={$width}
        height={$height}
        style="width: {$width}px; height: {$height}px;"
></canvas>
<svelte:window on:resize|passive={handleResize}/>
<slot dimensions={{width: $width, height: $height}} tick={tick}></slot>
<style>
</style>
