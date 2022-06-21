import type {LocalSession, Player, Session} from "../store/session";
import {SessionState, SessionType} from "../store/session";

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
    return fetch("/pong/api/create_session", {method: 'POST'}).then(res => res.json())
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function joinNetworkSession(sessionId): Promise<Session> {
    return fetch("/pong/api/join_session", {method: 'POST', body: JSON.stringify({session_id: sessionId})}).then(res => res.json())
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function watchNetworkSession(sessionId): Promise<Session> {
    return fetch("/pong/api/watch_session", {method: 'POST', body: JSON.stringify({session_id: sessionId})}).then(res => res.json())
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

export default {
    createLocalSession,
    createNetworkSession,
    joinNetworkSession,
    watchNetworkSession
}
