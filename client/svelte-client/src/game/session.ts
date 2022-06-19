import {get, Readable, readable, writable, derived} from "svelte/store";

export enum SessionState {
    PENDING = 'PENDING', RUNNING = 'RUNNING', CLOSED = 'CLOSED'
}

export enum SessionType {
    LOCAL = 'LOCAL', HOST = 'HOST', PEER = 'PEER', OBSERVER = 'OBSERVER'
}

export type Player = {
    id: string
}

export type Observer = {
    id: string
}

export type LocalSession = {
    state: SessionState
}

export type NetworkSession = {
    session_id: string,
    state: SessionState,
    players: Player[],
    observers: Observer[]
}

export type Session = LocalSession | NetworkSession;

export type Input = {
    input: 'UP' | 'DOWN',
    obj_id: number,
    player: number
}

enum InputMethod {
    UNDEFINED, KEYBOARD, NETWORK
}

const keysPressed: Readable<string[]> = readable([], function(set) {
    let keys = [];

    const onKeydown = ({key}) => {
        if (keys.includes(key)) {
            return;
        }
        keys = [...keys, key];
        set(keys);
    }
    const onKeyup = ({key}) => {
        if (!keys.includes(key)) {
            return;
        }
        keys = keys.filter(k => k !== key);
        set(keys);
    }

    document.addEventListener('keydown', onKeydown);
    document.addEventListener('keyup', onKeyup);

    return () => {
        document.removeEventListener('keydown', onKeydown);
        document.removeEventListener('keyup', onKeyup);
    }
})

export const localSessionInputs = derived(
    keysPressed,
    $keysPressed => {
        return $keysPressed.map((key): Input => {
            switch(key.toLowerCase()) {
                case 'w':
                    return {input: 'UP', obj_id: 0, player: 1};
                case 's':
                    return {input: 'DOWN', obj_id: 0, player: 1}
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

const inputMethod = writable(InputMethod.UNDEFINED)

export const sessionInputs = derived([inputMethod, localSessionInputs], ([$inputMethod, $localSessionInputs]) => {
    switch($inputMethod) {
        case InputMethod.KEYBOARD:
            return $localSessionInputs
        default:
            return []
    }
});

export type SessionStore = {
    session?: Session,
    sessionType?: SessionType
}

export const sessionContext = Symbol();

function initialValue(): SessionStore {
    return {
        session: null,
        sessionType: null
    }
}

function makeSessionStore() {
    const {subscribe, set, update} = writable(initialValue());

    return {
        subscribe,
        createLocalSession: () => {
            inputMethod.set(InputMethod.KEYBOARD);
            update(() => ({
                session: {state: SessionState.RUNNING},
                sessionType: SessionType.LOCAL
            }))
        },
        createNetworkSession: () => createSession().then(session => update(() => ({
            session,
            sessionType: SessionType.HOST
        }))),
        joinNetworkSession: (sessionId) => joinSession(sessionId).then(session => update(() => ({
            session,
            sessionType: SessionType.PEER,
        }))),
        watchNetworkSession: (sessionId) => watchSession(sessionId).then(session => update(() => ({
            session,
            sessionType: SessionType.OBSERVER
        }))),
        reset: () => set(initialValue())
    }
}

async function createSession(): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2_000)
    });
    return {
        session_id: "a",
        state: SessionState.PENDING,
        players: [],
        observers: []
    }
}

async function joinSession(sessionId): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
    });
    return {
        session_id: sessionId,
        state: SessionState.PENDING,
        players: [],
        observers: []
    }
}

async function watchSession(sessionId): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
    });
    return {
        session_id: sessionId,
        state: SessionState.RUNNING,
        players: [],
        observers: []
    }
}

export const sessionStore = makeSessionStore();
