import {get, Readable, readable, writable, derived} from "svelte/store";
import {keysPressed} from "./io";
import {createEventDispatcher, onDestroy} from "svelte";
import api from "../api/session";

export enum SessionState {
    PENDING = 'PENDING', RUNNING = 'RUNNING', CLOSED = 'CLOSED'
}

export enum SessionType {
    LOCAL = 'LOCAL', HOST = 'HOST', PEER = 'PEER', OBSERVER = 'OBSERVER'
}

export type Player = {
    id: string,
    nr: number
}

export type Observer = {
    id: string
}

export type LocalSession = {
    session_id: string,
    state: SessionState,
    type: SessionType.LOCAL
}

export type NetworkSession = {
    session_id: string,
    type: SessionType.HOST | SessionType.PEER | SessionType.OBSERVER,
    state: SessionState,
    players: Player[],
    you: Player
}

export type Session = LocalSession | NetworkSession;

export type Input = {
    input: 'UP' | 'DOWN',
    obj_id: number,
    player: number
}

export type InputEventPayload = {
    session_id: string,
    inputs: Input[],
    player_id: string,
    player_nr: number,
    ts: number,
}

const player1KeyboardInputs = derived(
    keysPressed,
    $keysPressed => {
        return $keysPressed.map((key): Input => {
            switch(key.toLowerCase()) {
                case 'w':
                    return {input: 'UP', obj_id: 0, player: 1};
                case 's':
                    return {input: 'DOWN', obj_id: 0, player: 1}
                default:
                    return null
            }
        }).filter(it => !!it);
    }
)

const player2KeyboardInputs = derived(
    keysPressed,
    $keysPressed => {
        return $keysPressed.map((key): Input => {
            switch(key.toLowerCase()) {
                case 'arrowup':
                    return {input: 'UP', obj_id: 1, player: 2}
                case 'arrowdown':
                    return {input: 'DOWN', obj_id: 1, player: 2}
                default:
                    return null
            }
        }).filter(it => !!it);
    }
)

const sessionEvents = (session: Session) => readable([], function(set) {
    const websocket = writable<WebSocket>(null);
    api.createEventWebsocket(session).then(ws => {
        websocket.set(ws);
    });

    const events = writable([]);
    derived(websocket, ($websocket: WebSocket) => {
        if (!websocket) {
            return;
        }
        $websocket.onmessage = event => {
            events.set([...get(events), event])
        }
    });

    const interval = setInterval(() => {
        set(get(events));
    }, 0)

    return () => {
        clearInterval(interval);
    }
})

const inputEvents = (session: Session): Readable<unknown[]> => derived(sessionEvents(session), ([$sessionEvents]) => $sessionEvents.filter(({input}) => input === 'topic'));

export const sessionInputs = (session: Session) => readable([], function(setInputs) {
    let player1Inputs = writable([]);
    let player2Inputs = writable([]);
    if (session.type === SessionType.LOCAL) {
        const p1Sub = player1KeyboardInputs.subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        const p2Sub = player2KeyboardInputs.subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
            onDestroy(p1Sub);
            onDestroy(p2Sub);
        }
    }
    if (session.type === SessionType.HOST) {
        const p1Sub = player1KeyboardInputs.subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        const p2Sub = inputEvents(session).subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
            onDestroy(p1Sub);
            onDestroy(p2Sub);
        }
    }
    if (session.type === SessionType.PEER) {
        const p1Sub = inputEvents(session).subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        const p2Sub = player2KeyboardInputs.subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
            onDestroy(p1Sub);
            onDestroy(p2Sub);
        }
    }
    if (session.type === SessionType.OBSERVER) {
        const events = inputEvents(session);
        const p1Sub = events.subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        const p2Sub = events.subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
            onDestroy(p1Sub);
            onDestroy(p2Sub);
        }
    }
    throw new Error(`unknown session type ${session.type}`)
})

export const sessionStore = writable<Session>(null);
