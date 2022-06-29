<script lang="ts">
    import {FieldWrapper} from "wasm-app";
    import {createEventDispatcher, onMount, setContext} from "svelte";
    import {drawObjects} from "../store/render";
    import {engineCanvas, engineCtx, height, pixelRatio, props, renderContext, width} from "../store/engine";
    import type {Input} from "../store/model/input";
    import type {GameObject, Session} from "../store/model/session";
    import {SessionType} from "../store/model/session";

    export let inputs: Input[] = []
    export let objects: GameObject[] = []
    export let session: Session;

    export let killLoopOnError = true;
    export let debug = false;

    const dispatch = createEventDispatcher();

    let canvas: any;
    let ctx: any;
    let frame: number;
    let listeners = [];

    let renderOnly = false;

    $: if(session) {
        renderOnly = session.type === SessionType.PEER || session.type === SessionType.OBSERVER;
    }

    onMount(() => {
        ctx = canvas.getContext('2d');
        engineCtx.set(ctx)
        engineCanvas.set(engineCanvas);
        width.set(canvas.width);
        height.set(canvas.height);
        // field.set_dimensions(canvas.width, canvas.height);

        // setup entities
        listeners.forEach(async entity => {
            if (entity.setup) {
                let p = entity.setup($props);
                if (p && p.then) await p;
            }
            entity.ready = true;
        });

        return createLoop((elapsed, dt) => {
            tick(dt);
            render(objects, dt);
        });
    })

    setContext(renderContext, {
        add (fn) {
            this.remove(fn);
            listeners.push(fn);
        },
        remove (fn) {
            const idx = listeners.indexOf(fn);
            if (idx >= 0) listeners.splice(idx, 1);
        }
    });

    function createLoop (fn) {
        let elapsed = 0;
        let lastTime = performance.now();
        (function loop() {
            frame = requestAnimationFrame(loop);
            const beginTime = performance.now();
            const dt = (beginTime - lastTime) / 1000;
            lastTime = beginTime;
            elapsed += dt;
            fn(elapsed, dt);
        })();
        return () => {
            cancelAnimationFrame(frame);
        };
    }

    function tick(dt) {
        dispatch('tick', [dt]);
    }

    function render(objects, dt) {
        const [canvas_width, canvas_height] = [canvas.width, canvas.height];
        ctx.clearRect(0, 0, canvas_width, canvas_height);
        drawObjects(ctx, objects, [canvas_width, canvas_height], debug);

        listeners.forEach(entity => {
            try {
                if (entity.mounted && entity.ready && entity.render) {
                    entity.render($props, dt);
                }
            } catch (err) {
                console.error(err);
                if (killLoopOnError) {
                    cancelAnimationFrame(frame);
                    console.warn('Animation loop stopped due to an error');
                }
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
        width={$width * $pixelRatio}
        height={$height * $pixelRatio}
        style="width: {$width}px; height: {$height}px;"
></canvas>
<svelte:window on:resize|passive={handleResize}/>
<slot></slot>

<style>
</style>
