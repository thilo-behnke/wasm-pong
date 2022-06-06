import { FieldWrapper, GameObject } from "wasm-app";

const GRID_COLOR = "#CCCCCC";

let field = FieldWrapper.new();
const width = field.width();
const height = field.height();

const canvas = document.getElementById('wasm-app-canvas');
canvas.height = height
canvas.width = width

const ctx = canvas.getContext('2d');

let framesLastSecond = [];
let lastFpsUpdate = 0;
const FPS_UPDATE_THRESHOLD = 1000;
let fps = 0;

let isInitial = true;
let lastUpdate = 0;
let paused = false;
let resetRequested = false;
let debug = false;
let keysDown = new Set();
let actions = [];

let networkSession = null;
let player = null;
let websocket = null;
let isHost = true;

let events = [];

const renderLoop = () => {
    if (resetRequested) {
        resetRequested = false;
        reset();
        return;
    }
    actions = getInputActions().filter(it => {
        if (!networkSession) {
            return it
        }
        if (networkSession && isHost) {
            return it.player === 1;
        }
        return it.player === 2;
    });
    if (paused) {
        requestAnimationFrame(renderLoop);
        return;
    }
    tick();
    requestAnimationFrame(renderLoop);
}

const tick = () => {
    const now = Date.now();
    let update;
    if (lastUpdate === 0) {
        update = 0.01;
        lastUpdate = now
    } else {
        const diff = now - lastUpdate;
        lastUpdate = now;
        update = diff / 1000;
    }
    if (debug) {
        framesLastSecond.push(now)
        framesLastSecond = framesLastSecond.filter(f => now - f <= 1000);
        if (lastFpsUpdate === 0 || now - lastFpsUpdate >= FPS_UPDATE_THRESHOLD) {
            lastFpsUpdate = now;
            fps = framesLastSecond.length;
        }
    }

    let objects;
    if (!networkSession) {
        field.tick(actions, update);
        objects = JSON.parse(field.objects());
    } else if (isHost) {
        // Would mean that input events would get lost if latency is higher than 100 ms.
        const peerInputEvents = events.filter(e => e.topic === "input").filter(it => it.msg.player !== player.id)
        const lastPeerInputEvents = peerInputEvents.length ? [peerInputEvents[peerInputEvents - 1]] : []
        const allActions = [
            ...actions, ...lastPeerInputEvents
        ];
        console.warn({allActions})
        field.tick(allActions, update);
        objects = JSON.parse(field.objects());
        sendEvents([...getInputEvents(), ...getMoveEvents(objects)])
    } else {
        const moveEventsByObj = events.filter(e => e.topic === "move").reduce((acc, moveEvent) => {
            const {id: objId} = moveEvent.msg;
            if (!acc[objId]) {
                acc[objId] = [];
            }
            acc[objId].push(moveEvent);
            return acc;
        }, {});
        const latestMoveEvents = Object.entries(moveEventsByObj)
            .map(([_, moveEvents]) => moveEvents[moveEvents.length - 1]);
        objects = latestMoveEvents.map(({msg}) => msg);
        sendEvents(getInputEvents())
    }
    render(objects);
}

const getMoveEvents = objects => {
    return objects.map(o => ({session_id: networkSession.hash, topic: 'move', msg: JSON.stringify({...o, session_id: networkSession.hash, ts: Date.now()})}));
}

const getInputEvents = () => {
    const inputEvents = actions.map(({input}) => ({msg: JSON.stringify({inputs: [input], player: player.id, session_id: networkSession.hash, ts: Date.now()}), session_id: networkSession.hash, topic: 'input'}));
    if (inputEvents.length) {
        return inputEvents;
    }
    const noInputs = {inputs: [], obj_id: isHost ? 0 : 1, player: isHost ? 1 : 2 }
    return [{msg: JSON.stringify({input: noInputs, player: player.id, session_id: networkSession.hash, ts: Date.now()}), session_id: networkSession.hash, topic: 'input'}];
}

const sendEvents = events => {
    const eventWrapper = {session_id: networkSession.hash, events};
    if (eventWrapper.events.length) {
        websocket.send(JSON.stringify(eventWrapper));
    }
}

const render = objects => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawObjects(objects);
}

const reset = () => {
    framesLastSecond = [];
    lastFpsUpdate = 0;
    fps = 0;

    resetRequested = false;
    isInitial = true;
    lastUpdate = 0;
    paused = false;
    debug = false;
    keysDown = new Set();
    actions = [];

    field = FieldWrapper.new();

    networkSession = null;
    isHost = true;
    if (websocket) {
        websocket.close();
    }
    websocket = null;

    const context = canvas.getContext('2d');
    context.clearRect(0, 0, canvas.width, canvas.height);
}

window.WASM_PONG = {}
window.WASM_PONG.width = width
window.WASM_PONG.height = height

window.WASM_PONG.createOnlineSession = () => {
    resetRequested = true;
    fetch("http://localhost:4000/create_session", {method: 'POST'}).then(res => res.json()).then(({data: session}) => {
        console.log("Created session:")
        console.log(session)
        networkSession = session.session
        player = session.player
        isHost = true;
        const session_display_tag = document.getElementById("network_session");
        session_display_tag.style.display = 'block';
        session_display_tag.innerHTML = JSON.stringify(session)

        websocket = new WebSocket(`ws://localhost:4000/ws?session_id=${session.session.hash}&connection_type=host`)
        websocket.onmessage = (event) => {
            addEvents(event);
        }
        waitForWebsocket(10, renderLoop)
    }).catch(err => {
        console.error(`Failed to create session: ${err}`)
    })
}

window.WASM_PONG.joinOnlineSession = () => {
    if (!isInitial) {
        resetRequested = true;
        setTimeout(() => {
            window.WASM_PONG.joinOnlineSession()
        })
        return;
    }

    const sessionId = document.getElementById('join-online-input').value
    fetch(`http://localhost:4000/join_session`, {method: 'POST', body: JSON.stringify({session_id: sessionId})}).then(res => res.json()).then(({data: session}) => {
        console.log("Joined session:")
        console.log(session);
        networkSession = session.session
        player = session.player
        isHost = false
        const session_display_tag = document.getElementById("network_session");
        session_display_tag.style.display = 'block';
        session_display_tag.innerHTML = JSON.stringify(session)
        // Field will be instrumented by only drawing the objects for the peer, e.g. collision detection will happen at the host.
        field = null;

        websocket = new WebSocket(`ws://localhost:4000/ws?session_id=${session.session.hash}&connection_type=peer`)
        websocket.onmessage = (event) => {
            addEvents(event);
        }
        waitForWebsocket(10, renderLoop)
    }).catch(err => {
        console.error(`Failed to create session: ${err}`)
    })
}

const waitForWebsocket = (retries, cb) => {
    if (retries <= 0) {
        console.error("Websocket not established successfully")
        return
    }
    if(websocket.readyState !== 1) {
        setTimeout(() => {
            waitForWebsocket(retries - 1, cb)
        }, 100)
    } else {
        cb()
    }
}

const addEvents = wsEvent => {
    const gameEvents = JSON.parse(wsEvent.data);
    gameEvents.forEach(gameEvent => {
        const deserialized = {...gameEvent, msg: JSON.parse(gameEvent.msg)};
        events.push(deserialized);
        events.slice(events.length - 100, events.length)
    })
}

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

window.WASM_PONG.resetGame = () => {
    reset()
    document.getElementById("pause-btn").disabled = true;
    document.getElementById("resume-btn").disabled = false;
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

const drawObjects = objects => {
    objects.forEach(obj => {
        ctx.beginPath();
        ctx.strokeStyle = GRID_COLOR;

        const obj_y = height - obj.y;
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

const drawLine = (ctx, from_x, from_y, to_x, to_y, color) => {
    ctx.beginPath();
    ctx.moveTo(from_x, from_y);
    ctx.strokeStyle = color;
    ctx.lineTo(to_x, to_y);
    ctx.stroke();
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
                return {input: 'UP', obj_id: 0, player: 1}
            case 'KeyS':
                return {input: 'DOWN', obj_id: 0, player: 1}
            case 'ArrowUp':
                return {input: 'UP', obj_id: 1, player: 2}
            case 'ArrowDown':
                return {input: 'DOWN', obj_id: 1, player: 2}
            default:
                return null
        }
    }).filter(it => !!it);
}

listenToKeys();
render(JSON.parse(field.objects()));
requestAnimationFrame(renderLoop);
