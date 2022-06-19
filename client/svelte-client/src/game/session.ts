import {readable, writable} from "svelte/store";

export enum SessionState {
    PENDING, RUNNING, CLOSED
}

export enum SessionType {
    HOST, PEER, OBSERVER
}

export type Player = {
    id: string
}

export type Observer = {
    id: string
}

export type Session = {
    session_id: string,
    state: SessionState,
    players: Player[],
    observers: Observer[]
}

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
        createSession: () => createSession().then(session => update(() => ({session, sessionType: SessionType.HOST}))),
        joinSession: () => joinSession().then(session => update(() => ({session, sessionType: SessionType.PEER}))),
        watchSession: () => watchSession().then(session => update(() => ({session, sessionType: SessionType.OBSERVER}))),
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
        state: SessionState.CLOSED,
        players: [],
        observers: []
    }
}

async function joinSession(): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
    });
    return {
        session_id: "a",
        state: SessionState.CLOSED,
        players: [],
        observers: []
    }
}

async function watchSession(): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
    });
    return {
        session_id: "a",
        state: SessionState.CLOSED,
        players: [],
        observers: []
    }
}

export const sessionStore = makeSessionStore();
