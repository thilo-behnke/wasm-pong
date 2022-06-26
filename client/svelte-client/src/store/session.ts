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
        console.log("ws successfully established: ", ws)

        ws.onopen = () => {
            console.debug("ws successfully opened")
        }
        ws.onmessage = event => {
            console.debug("Received event: ", event)
            let events = JSON.parse(event.data);
            // TODO: Hotfix, would be better to have clean serialization in the backend...
            events = events.map(({event, ...rest}) => ({...rest, event: JSON.parse(event)}))
            console.debug("Parsed events: ", events)
            set(events);
            set([]);
        }
        ws.onerror = err => {
            console.error("ws error: ", err)
        }
        ws.onclose = event => {
            console.error("ws closed: ", event)
        }

        websocket.set(ws);
    });

    const interval = setInterval(() => {
    }, 0)

    set([]);

    return () => {
        get(websocket).close();
        clearInterval(interval);
    }
})

export const networkSessionStateEvents = (session: NetworkSession): Readable<unknown[]> => derived(networkEvents(session), $sessionEvents => {
    console.log("test")
    const sessionEvents = $sessionEvents.filter(({topic}) => topic === 'session').map(({event}) => event);
    if (!sessionEvents.length) {
        return [];
    }
    const latestSessionEvent = sessionEvents[sessionEvents.length-1];
    const currentSession = get(sessionStore) as NetworkSession;
    const session = {...latestSessionEvent.session, you: currentSession.you, type: currentSession.type}
    console.debug("updating current session: ", session)
    sessionStore.set(session);
    return sessionEvents;
});

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
