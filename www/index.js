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
        .map(([id, x, y, shape]) => {
            const shape_1 = shape >> 8;
            const shape_2 = shape & (2**8 - 1);
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
