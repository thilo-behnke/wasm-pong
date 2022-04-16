import * as wasm from "wasm-app";
import { Field, GameObject } from "wasm-app";
import { memory } from "wasm-app/rust_wasm_bg";

// ...

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const field = Field.new();
const width = field.width;
const height = field.height;

const canvas = document.getElementById('wasm-app-canvas');
canvas.height = height
canvas.width = width

console.log(({width, height}))

const ctx = canvas.getContext('2d');

const renderLoop = () => {
    field.tick([]);

    drawField();
    drawObjects();
    requestAnimationFrame(renderLoop);
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
        .map(([id, x, y, _]) => ({id, x, y}));
    return objects;
}

drawField();
drawObjects();
requestAnimationFrame(renderLoop);
