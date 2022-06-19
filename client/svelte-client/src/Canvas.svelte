<script lang="ts">
    import { FieldWrapper } from "wasm-app";
    import {onMount} from "svelte";
    import {get, writable} from "svelte/store";
    const field = FieldWrapper.new();
    let canvas: any;
    let ctx: any;

    const GRID_COLOR = "#CCCCCC";
    const width = writable(field.width());
    const height = writable(field.height());
    const pixelRatio = writable(window.devicePixelRatio);

    let debug = false;
    let fps = 0;

    onMount(() => {
        ctx = canvas.getContext('2d');
        requestAnimationFrame(renderLoop);
    })

    function renderLoop(dt) {
        tick();
        requestAnimationFrame(renderLoop);
    }

    function tick() {
        field.tick([], 0.01);
        let objects = JSON.parse(field.objects());
        render(objects);
    }

    function render(objects) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        drawObjects(objects);
    }

    const drawObjects = objects => {
        const canvas_height = get(height);
        objects.forEach(obj => {
            ctx.beginPath();
            ctx.strokeStyle = GRID_COLOR;

            const obj_y = canvas_height - obj.y;
            const orientation_y = obj.orientation_y * -1;
            const vel_y = obj.vel_y * -1;

            // rect
            if (obj.shape_param_2) {
                ctx.moveTo(obj.x, obj_y)
                ctx.arc(obj.x, obj_y, 10, 0, 2 * Math.PI);
                ctx.rect(obj.x - obj.shape_param_1 / 2, obj_y - obj.shape_param_2 / 2, obj.shape_param_1, obj.shape_param_2);
            }
            // circle
            else {
                ctx.moveTo(obj.x, obj_y);
                ctx.arc(obj.x, obj_y, obj.shape_param_1, 0, 2 * Math.PI);
            }
            ctx.stroke();

            if (debug) {
                // velocity
                drawLine(ctx, obj.x, obj_y, obj.x + obj.vel_x * 20, obj_y + vel_y * 20, 'red')
                // orientation
                drawLine(ctx, obj.x, obj_y, obj.x + obj.orientation_x * 20, obj_y + orientation_y * 20, 'blue')
                ctx.fillText(`[x: ${obj.x}, y: ${obj_y}]`, obj.x + 10, obj_y)
            }
        })
        if (debug) {
            const fpsPosX = width - 50;
            const fpsPosY = 20;
            ctx.rect(fpsPosX, fpsPosY, 100, 20)
            ctx.fillText(`fps: ${fps}`, fpsPosX, fpsPosY)
        }
    }

    function drawLine(ctx, from_x, from_y, to_x, to_y, color) {
        ctx.beginPath();
        ctx.moveTo(from_x, from_y);
        ctx.strokeStyle = color;
        ctx.lineTo(to_x, to_y);
        ctx.stroke();
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
