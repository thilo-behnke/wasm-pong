import {derived, get, readable, Readable, writable} from "svelte/store";
import {keysPressed} from "./io";
import {onDestroy} from "svelte";
import api from "../api/session";
import type {NetworkSession, Session} from "./model/session";
import {isLocalSession, SessionType} from "./model/session";

export type Input = {
    input: 'UP' | 'DOWN',
    obj_id: number,
    player: number
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

const networkEvents = (session: NetworkSession) => readable([], function(set) {
    const websocket = writable<WebSocket>(null);

    console.log("creating ws to receive/send websocket events for session: ", JSON.stringify(session))
    api.createEventWebsocket(session).then(ws => {
        websocket.set(ws);
    });

    const events = writable([]);
    derived(websocket, ($websocket: WebSocket) => {
        if (!websocket) {
            return;
        }
        console.log("ws ready, registering message handler")
        $websocket.onmessage = event => {
            console.debug("Received event: ", event)
            events.set([...get(events), event])
        }
        $websocket.onerror = err => {
            console.error("ws error: ", err)
        }
        $websocket.onclose = event => {
            console.error("ws closed: ", event)
        }
    });

    const interval = setInterval(() => {
        set(get(events));
    }, 0)

    set([]);

    return () => {
        get(websocket).close();
        clearInterval(interval);
    }
})

export const networkSessionStateEvents = (session: NetworkSession): Readable<unknown[]> => derived(networkEvents(session), $sessionEvents => $sessionEvents.filter(({topic}) => topic === 'session'));

const networkInputEvents = (session: NetworkSession): Readable<unknown[]> => derived(networkEvents(session), $sessionEvents => $sessionEvents.filter(({topic}) => topic === 'input'));
export const sessionInputs = (session: Session) => readable([], function(setInputs) {
    let player1Inputs = writable([]);
    let player2Inputs = writable([]);
    if (isLocalSession(session)) {
        player1KeyboardInputs.subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        player2KeyboardInputs.subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
        }
    }

    if (session.type === SessionType.HOST) {
        player1KeyboardInputs.subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        networkInputEvents(session).subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
        }
    }
    if (session.type === SessionType.PEER) {
        networkInputEvents(session).subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        player2KeyboardInputs.subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
        }
    }
    if (session.type === SessionType.OBSERVER) {
        const events = networkInputEvents(session);
        events.subscribe(inputs => {
            player1Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        events.subscribe(inputs => {
            player2Inputs.set(inputs)
            setInputs([...get(player1Inputs), ...get(player2Inputs)])
        })
        return () => {
        }
    }
    throw new Error(`unknown session type ${session.type}`)
})

export const sessionStore = writable<Session>(null);
