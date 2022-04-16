import * as wasm from "wasm-app";
import { Field, GameObject } from "wasm-app";
import { memory } from "wasm-app/rust_wasm_bg";

const GRID_COLOR = "#CCCCCC";

const field = Field.new();
const width = field.width;
const height = field.height;

const canvas = document.getElementById('wasm-app-canvas');
canvas.height = height
canvas.width = width

const ctx = canvas.getContext('2d');

let keysDown = new Set();

const renderLoop = () => {
    let actions = getInputActions();
    field.tick(actions);

    render();
    requestAnimationFrame(renderLoop);
}

const render = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawField();
    drawObjects();
}

const drawField = () => {
    ctx.beginPath();

    ctx.strokeStyle = GRID_COLOR;
    ctx.rect(1, 1, field.width - 2, field.height - 2);

    ctx.stroke();
}

const drawObjects = () => {
    const objects = getObjects();
    ctx.beginPath();

    objects.forEach(obj => {
        ctx.strokeStyle = GRID_COLOR;
        ctx.moveTo(obj.x - 10, obj.y)
        ctx.lineTo(obj.x, obj.y)
    })

    ctx.stroke();
}

const getObjects = () => {
    const objectsPtr = field.objects();
    const objects = new Uint16Array(memory.buffer, objectsPtr, 3 * 4) // player1, player2, ball
        .reduce((acc, val) => {
            if (!acc.length) {
                return [[val]]
            }
            const last = acc[acc.length - 1]
            if (last.length === 4) {
                return [...acc, [val]]
            }
            return [...acc.slice(0, -1), [...last, val]]
        }, [])
        .map(([id, x, y, _]) => ({id, x, y: height - y}));
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
