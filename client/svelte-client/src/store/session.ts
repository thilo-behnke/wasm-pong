import {derived, get, Readable, readable, Unsubscriber, writable} from "svelte/store";
import {keysPressed} from "./io";
import api from "../api/session";
import session from "../api/session";
import type {
    GameObject,
    Heartbeat,
    LocalSession,
    Message,
    NetworkSession,
    Session,
    SessionSnapshot
} from "./model/session";
import {isLocalSession, isNetworkSession, MessageType, SessionState, SessionType} from "./model/session";
import type {NetworkStore} from "./network";
import type {GameEventWrapper, InputEventPayload, InputEventWrapper, SessionEventPayload} from "./model/event";
import {isInputEvent, isMoveEvent} from "./model/event";
import {getPlayerKeyboardInputs, playerKeyboardInputs} from "./input";
import type {Subscriber} from "svelte/types/runtime/store";
import {combined} from "./utils";
import type {Input} from "./model/input";

const sessionStore = writable<Session>(null)

function createNetworkEvents() {
    const {subscribe, set, update} = writable<GameEventWrapper[]>([]);

    const websocket = writable<WebSocket>(null);
    const sessionId = writable<string>(null);
    const playerId = writable<string>(null);
    const lastSnapshot = writable<SessionSnapshot>(null);

    const unsubscribeSession = sessionStore.subscribe(session => {
        if (!session || isLocalSession(session)) {
            return;
        }
        if (get(sessionId) === session.session_id) {
            return;
        }
        sessionId.set(session.session_id);
        playerId.set(session.you.id);
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
            }
            ws.onerror = err => {
                console.error("ws error: ", err)
            }
            ws.onclose = event => {
                console.error("ws closed: ", event)
            }

            websocket.set(ws);
        });
    })

    const interval = setInterval(() => {
        const cachedSessionId = get(sessionId);
        if (!cachedSessionId) {
            return;
        }
        const last = get(lastSnapshot);
        const now = Date.now();
        if (last && now - last.ts < 1_000) {
            return
        }
        console.debug("sending heartbeat")
        const heartbeat: Message = {msg_type: MessageType.Heartbeat, payload: {session_id: cachedSessionId, player_id: get(playerId), ts: now}};
        sendMessage(heartbeat);
    }, 1_000)

    function sendMessage(message: Message) {
        const ws = get(websocket);
        if (!ws) {
            return;
        }
        console.debug("producing message to ws: ", message);
        // TODO: Hotfix, double serialize to ease deserialization on server.
        ws.send(JSON.stringify({msg_type: message.msg_type, payload: JSON.stringify(message.payload)}));
    }

    function produce(snapshot: SessionSnapshot) {
        lastSnapshot.set(snapshot);
        sendMessage({msg_type: MessageType.Snapshot, payload: snapshot});
    }

    const customSubscribe = (run: Subscriber<GameEventWrapper[]>, invalidate): Unsubscriber => {
        const unsubscribe = subscribe(run, invalidate);
        return () => {
            unsubscribeSession();
            clearInterval(interval);
            unsubscribe();
        }
    }

    return {
        subscribe: customSubscribe,
        produce
    }
}

export type NetworkEventStore = Readable<GameEventWrapper[]> & {
    produce: (snapshot: SessionSnapshot) => void
}

export const networkEvents: NetworkEventStore = createNetworkEvents();

export const networkSessionStateEvents = derived(networkEvents, $sessionEvents => {
    const sessionEvents = $sessionEvents.filter(({topic}) => topic === 'session').map(({event}) => event);
    if (!sessionEvents.length) {
        return [];
    }
    const latestSessionEvent = sessionEvents[sessionEvents.length - 1] as SessionEventPayload;
    const currentSession = get(sessionStore) as NetworkSession;
    const session: Session = {
        ...(latestSessionEvent.session as NetworkSession),
        you: currentSession.you,
        type: currentSession.type
    }
    console.debug("updating current session: ", session)
    sessionStore.set(session);
    return sessionEvents;
});

export const networkMoveEvents = derived(networkEvents, $sessionEvents => {
    const moveEvents = $sessionEvents.filter(isMoveEvent).map(({event}) => event);
    if (!moveEvents.length) {
        return [];
    }
    // TODO: How to know number of objects?
    return moveEvents.slice(moveEvents.length - 7)
})

const networkInputEvents = derived([networkEvents, sessionStore], ([$sessionEvents, $sessionStore]) => $sessionEvents.filter(wrapper => {
    if (!isInputEvent(wrapper)) {
        return false;
    }
    return wrapper.event.player_id !== ($sessionStore as NetworkSession).you.id
}).map(({event}) => event as InputEventPayload));

const getPlayerNetworkInputEvents = (player_nr: number): Readable<Input[]> => derived(networkInputEvents, $networkInputEvents => {
    const session = get(sessionStore);
    if (!isNetworkSession(session)) {
        return [] as Input[];
    }
    const player = session.players.find(({nr}) => player_nr === nr);
    if (!player) {
        return [] as Input[];
    }
    const inputEvents = $networkInputEvents.filter(({player_id}) => player.id === player_id);
    if (!inputEvents.length) {
        return [] as Input[];
    }
    return inputEvents[inputEvents.length - 1].inputs
});

export const sessionInputs = readable([], function (setInputs) {
    setInputs([]);

    const unsubscribe = sessionStore.subscribe(session => {
        return getInputStore(session).subscribe(input => {
            setInputs(input);
        });
    });

    return () => {
        unsubscribe();
    }
})

const getInputStore = (session: Session): Readable<Input[]> => {
    if (isLocalSession(session)) {
        return playerKeyboardInputs;
    }
    const sessionType = session.type;
    if (sessionType === SessionType.HOST) {
        return combined(getPlayerKeyboardInputs(1), getPlayerNetworkInputEvents(2));
    }
    if (sessionType === SessionType.PEER) {
        return combined(getPlayerNetworkInputEvents(1), getPlayerKeyboardInputs(2));
    }
    if (sessionType === SessionType.OBSERVER) {
        return combined(getPlayerNetworkInputEvents(1), getPlayerNetworkInputEvents(2));
    }
    throw new Error(`unknown session type ${session.type}`)
}

export const localSession = () => readable<SessionStore>(null, function (set) {
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

export const networkSession = (type: SessionType.HOST | SessionType.PEER | SessionType.OBSERVER, sessionId?: string) => readable<SessionStore>(null, function (set) {
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
