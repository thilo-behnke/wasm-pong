<script lang="ts">
    import { FieldWrapper } from "wasm-app";
    import {onMount} from "svelte";
    import {get, writable} from "svelte/store";
    import {drawObjects} from "./render/render";
    const field = FieldWrapper.new();

    let canvas: any;
    let ctx: any;
    let frame: number;

    const width = writable(field.width());
    const height = writable(field.height());
    const pixelRatio = writable(window.devicePixelRatio);

    let debug = writable(false);
    let fps = 0;

    onMount(() => {
        ctx = canvas.getContext('2d');
        return createLoop((elapsed, dt) => {
            tick(dt);
            const objects = JSON.parse(field.objects());
            render(objects);
        });
    })

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
        field.tick([], dt);
        let objects = JSON.parse(field.objects());
        render(objects);
    }

    function render(objects) {
        const [canvas_width, canvas_height] = [canvas.width, canvas.height];
        ctx.clearRect(0, 0, canvas_width, canvas_height);
        drawObjects(ctx, objects, [canvas_width, canvas_height], get(debug));
    }

    function handleResize () {
        width.set(window.innerWidth);
        height.set(window.innerHeight);
        pixelRatio.set(window.devicePixelRatio);
    }
</script>

<canvas
        bind:this={canvas}
        width={$width * $pixelRatio}
        height={$height * $pixelRatio}
        style="width: {$width}px; height: {$height}px;"
></canvas>
<svelte:window on:resize|passive={handleResize} />

<style>

</style>
