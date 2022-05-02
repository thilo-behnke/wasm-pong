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

const drawObjects = () => {
    const objects = getObjects();
    ctx.beginPath();

    objects.forEach(obj => {
        ctx.strokeStyle = GRID_COLOR;

        // rect
        if (obj.shape_2) {
            ctx.moveTo(obj.x, obj.y)
            ctx.arc(obj.x, obj.y, 10, 0, 2 * Math.PI);
            ctx.rect(obj.x - obj.shape_1 / 2, obj.y - obj.shape_2 / 2, obj.shape_1, obj.shape_2);
        }
        // circle
        else {
            ctx.moveTo(obj.x, obj.y);
            ctx.arc(obj.x, obj.y, obj.shape_1, 0, 2 * Math.PI);
        }
    })

    ctx.stroke();
}

const getObjects = () => {
    const objectsPtr = field.objects();
    const objects = new Uint16Array(memory.buffer, objectsPtr, 3 * 5 + 4 * 5) // player1, player2, ball + 4x bounds
        .reduce((acc, val) => {
            if (!acc.length) {
                return [[val]]
            }
            const last = acc[acc.length - 1]
            if (last.length === 5) {
                return [...acc, [val]]
            }
            return [...acc.slice(0, -1), [...last, val]]
        }, [])
        .map(([id, x, y, shape_1, shape_2]) => {
            return {id, x, y: height - y, shape_1, shape_2};
        });
    return objects;
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
