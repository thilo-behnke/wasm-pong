import {derived, get, readable, writable} from "svelte/store";
import {keysPressed} from "./io";
import api from "../api/session";
import session from "../api/session";
import type {LocalSession, NetworkSession, Session} from "./model/session";
import {isLocalSession, SessionState, SessionType} from "./model/session";
import type {NetworkStore} from "./network";
import type {GameEventWrapper, SessionEventPayload} from "./model/event";

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

const networkEvents = readable<GameEventWrapper[]>([], function(set) {
    const websocket = writable<WebSocket>(null);
    const sessionId = writable<string>(null)

    const unsubscribe = sessionStore.subscribe(session => {
        if (!session || isLocalSession(session)) {
            return;
        }
        if (get(sessionId) === session.session_id) {
            return;
        }
        sessionId.set(session.session_id);
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

        set([]);

    })

    return () => {
        get(websocket).close();
        unsubscribe();
    }
})

export const networkSessionStateEvents = derived(networkEvents, $sessionEvents => {
    const sessionEvents = $sessionEvents.filter(({topic}) => topic === 'session').map(({event}) => event);
    if (!sessionEvents.length) {
        return [];
    }
    const latestSessionEvent = sessionEvents[sessionEvents.length-1] as SessionEventPayload;
    const currentSession = get(sessionStore) as NetworkSession;
    const session: Session = {...(latestSessionEvent.session as NetworkSession), you: currentSession.you, type: currentSession.type}
    console.debug("updating current session: ", session)
    sessionStore.set(session);
    return sessionEvents;
});

const networkInputEvents = derived(networkEvents, $sessionEvents => $sessionEvents.filter(({topic}) => topic === 'input'));
export const sessionInputs = readable([], function(setInputs) {
    let player1Inputs = writable([]);
    let player2Inputs = writable([]);
    setInputs([]);

    const unsubscribe = sessionStore.subscribe(session => {
        if (isLocalSession(session)) {
            const unsub1 = player1KeyboardInputs.subscribe(inputs => {
                player1Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            const unsub2 = player2KeyboardInputs.subscribe(inputs => {
                player2Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            return () => {
                unsub1();
                unsub2();
            }
        }

        if (session.type === SessionType.HOST) {
            const unsub1 = player1KeyboardInputs.subscribe(inputs => {
                player1Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            const unsub2 = networkInputEvents.subscribe(inputs => {
                player2Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            return () => {
                unsub1();
                unsub2();
            }
        }
        if (session.type === SessionType.PEER) {
            const unsub1 = networkInputEvents.subscribe(inputs => {
                player1Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            const unsub2 = player2KeyboardInputs.subscribe(inputs => {
                player2Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            return () => {
                unsub1();
                unsub2();
            }
        }
        if (session.type === SessionType.OBSERVER) {
            const events = networkInputEvents;
            const unsub1 = events.subscribe(inputs => {
                player1Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            const unsub2 = events.subscribe(inputs => {
                player2Inputs.set(inputs)
                setInputs([...get(player1Inputs), ...get(player2Inputs)])
            })
            return () => {
                unsub1();
                unsub2();
            }
        }
        throw new Error(`unknown session type ${session.type}`)
    })
})

const sessionStore = writable<Session>(null)

export const localSession = () => readable<SessionStore>(null, function(set) {
    const session: LocalSession = {session_id: "local", type: SessionType.LOCAL, state: SessionState.RUNNING};
    set({loading: true});
    setTimeout(() => {
        set({loading: false, session});
        sessionStore.set(session);
    }, 2_000);
})

export type SessionStore = NetworkStore & {
    session?: Session
}

export const networkSession = (type: SessionType.HOST | SessionType.PEER | SessionType.OBSERVER, sessionId?: string) => readable<SessionStore>(null, function(set) {
    function sessionCreator(fn) {
        set({loading: true});
        fn().then(session => {
            set({loading: false, session});
            sessionStore.set(session);
        }).catch(e => {
            set({loading: false, error: {value: e, at: performance.now()}});
            sessionStore.set(null);
        })
    }

    const unsubscribe = sessionStore.subscribe(session => {
        set({loading: false, session})
    })

    switch (type) {
        case SessionType.HOST:
            sessionCreator(() => api.createNetworkSession());
            break;
        case SessionType.PEER:
            sessionCreator(() => api.joinNetworkSession(sessionId));
            break;
        case SessionType.OBSERVER:
            sessionCreator(() => api.watchNetworkSession(sessionId));
            break;
        default:
            throw new Error("Unable to handle session type: " + type)
    }

    return () => {
        unsubscribe();
    }
})