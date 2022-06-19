import {readable, writable} from "svelte/store";

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
        createLocalSession: () => update(() => ({session: {state: SessionState.RUNNING}})),
        createNetworkSession: () => createSession().then(session => update(() => ({session, sessionType: SessionType.HOST}))),
        joinNetworkSession: (sessionId) => joinSession(sessionId).then(session => update(() => ({session, sessionType: SessionType.PEER}))),
        watchNetworkSession: (sessionId) => watchSession(sessionId).then(session => update(() => ({session, sessionType: SessionType.OBSERVER}))),
        reset: () => set(initialValue())
    }
}

async function createSession(): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
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
