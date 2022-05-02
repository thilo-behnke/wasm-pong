import * as wasm from "wasm-app";
import { FieldWrapper, GameObject } from "wasm-app";
import { memory } from "wasm-app/rust_wasm_bg";

const GRID_COLOR = "#CCCCCC";

const field = FieldWrapper.new();
const width = field.width();
const height = field.height();

const canvas = document.getElementById('wasm-app-canvas');
canvas.height = height
canvas.width = width

const ctx = canvas.getContext('2d');

let paused = false;
let debug = false;
let keysDown = new Set();
let actions = [];

const renderLoop = () => {
    actions = getInputActions();
    if (paused) {
        requestAnimationFrame(renderLoop);
        return;
    }
    tick();
    requestAnimationFrame(renderLoop);
}

const tick = () => {
    field.tick(actions);
    render();
}

const render = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawObjects();
}

window.WASM_PONG = {}
window.WASM_PONG.width = width
window.WASM_PONG.height = height

window.WASM_PONG.pauseGame = () => {
    paused = true;
    document.getElementById("pause-btn").disabled = true;
    document.getElementById("resume-btn").disabled = false;
    document.getElementById("tick-btn").disabled = false;
}

window.WASM_PONG.resumeGame = () => {
    paused = false;
    document.getElementById("pause-btn").disabled = false;
    document.getElementById("resume-btn").disabled = true;
    document.getElementById("tick-btn").disabled = true;
}


window.WASM_PONG.oneTick = () => {
    if (!paused) {
        return;
    }
    tick()
}

window.WASM_PONG.toggleDebug = () => {
    debug = !debug
}

const drawObjects = () => {
    const objects = getObjects();

    objects.forEach(obj => {
        ctx.beginPath();
        ctx.strokeStyle = GRID_COLOR;

        // rect
        if (obj.shape_param_2) {
            ctx.moveTo(obj.x, obj.y)
            ctx.arc(obj.x, obj.y, 10, 0, 2 * Math.PI);
            ctx.rect(obj.x - obj.shape_param_1 / 2, obj.y - obj.shape_param_2 / 2, obj.shape_param_1, obj.shape_param_2);
        }
        // circle
        else {
            ctx.moveTo(obj.x, obj.y);
            ctx.arc(obj.x, obj.y, obj.shape_param_1, 0, 2 * Math.PI);
        }
        ctx.stroke();

        if (debug) {
            // velocity
            drawLine(ctx, obj.x, obj.y, obj.x + obj.vel_x * 20, obj.y + obj.vel_y * 20, 'red')
            // orientation
            drawLine(ctx, obj.x, obj.y, obj.x + obj.orientation_x * 20, obj.y + obj.orientation_y * 20, 'blue')
        }
    })
}

const drawLine = (ctx, from_x, from_y, to_x, to_y, color) => {
    ctx.beginPath();
    ctx.moveTo(from_x, from_y);
    ctx.strokeStyle = color;
    ctx.lineTo(to_x, to_y);
    ctx.stroke();
}

const getObjects = () => {
    return JSON.parse(field.objects());
}

const listenToKeys = () => {
    const relevantKeys = ['ArrowUp', 'ArrowDown', 'KeyW', 'KeyS']
    document.addEventListener('keydown', (e) => {
        if (!relevantKeys.includes(e.code)) {
            return;
        }
        keysDown.add(e.code)
    })
    document.addEventListener('keyup', (e) => {
        if (!relevantKeys.includes(e.code)) {
            return;
        }
        keysDown.delete(e.code);
    })
}

const getInputActions = () => {
    return [...keysDown].map(key => {
        switch(key) {
            case 'KeyW':
                return {input: 'UP', obj_id: 0}
            case 'KeyS':
                return {input: 'DOWN', obj_id: 0}
            case 'ArrowUp':
                return {input: 'UP', obj_id: 1}
            case 'ArrowDown':
                return {input: 'DOWN', obj_id: 1}
            default:
                return null
        }
    }).filter(it => !!it);
}

listenToKeys();
render();
requestAnimationFrame(renderLoop);
