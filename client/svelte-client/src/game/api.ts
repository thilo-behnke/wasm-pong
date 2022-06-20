import type {LocalSession, Player, Session} from "./session";
import {SessionState, SessionType} from "./session";

async function createLocalSession(): Promise<LocalSession> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2_000)
    });
    return {
        session_id: "local_session",
        type: SessionType.LOCAL,
        state: SessionState.RUNNING
    }
}

async function createNetworkSession(): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2_000)
    });
    return {
        session_id: "a",
        type: SessionType.HOST,
        state: SessionState.PENDING,
        players: [],
        observers: []
    }
}

async function joinNetworkSession(sessionId): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
    });
    return {
        session_id: sessionId,
        type: SessionType.PEER,
        state: SessionState.PENDING,
        players: [],
        observers: []
    }
}

async function watchNetworkSession(sessionId): Promise<Session> {
    await new Promise((res) => {
        setTimeout(() => {
            res(null)
        }, 2000)
    });
    return {
        session_id: sessionId,
        type: SessionType.OBSERVER,
        state: SessionState.RUNNING,
        players: [],
        observers: []
    }
}


export default {
    createLocalSession,
    createNetworkSession,
    joinNetworkSession,
    watchNetworkSession
}
