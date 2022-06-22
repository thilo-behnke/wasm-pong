import type {LocalSession, Session} from "../store/session";
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
    return fetch("/pong/api/create_session", {method: 'POST', headers: [['Content-Type', 'application/json']]})
        .then(sessionResponseHandler)
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function joinNetworkSession(sessionId): Promise<Session> {
    return fetch("/pong/api/join_session", {method: 'POST', body: JSON.stringify({session_id: sessionId}), headers: [['Content-Type', 'application/json']]})
        .then(sessionResponseHandler)
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function watchNetworkSession(sessionId): Promise<Session> {
    return fetch("/pong/api/watch_session", {method: 'POST', body: JSON.stringify({session_id: sessionId}), headers: [['Content-Type', 'application/json']] })
        .then(sessionResponseHandler)
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function sessionResponseHandler(response: Response): Promise<any> {
    if(!response.ok) {
        return response.text().then(text => {
           return Promise.reject(`${response.status}: ${text}`)
        });
    }
    return response.json();
}

async function createEventWebsocket(session: Session): Promise<WebSocket> {
    return new Promise((res, rej) => {
        if (session.type === SessionType.LOCAL) {
            return rej("Websocket not allowed for local session!");
        }
        const url = `pong/ws?session_id=${session.session_id}&connection_type=${session.type.toLowerCase()}`;
        return createWebsocket(url);
    })
}

async function createWebsocket(path: string): Promise<WebSocket> {
    return new Promise((res, rej) => {
        const baseUrl = location.host.split(':')[0];
        const websocket = new WebSocket(`ws://${baseUrl}/${path}`);
        waitForWebsocket(websocket, 10, () => {
            return res(websocket)
        }, () => rej())
    })
}

const waitForWebsocket = (websocket, retries, success, fail) => {
    if (retries <= 0) {
        console.error("Websocket not established successfully")
        return
    }
    if(websocket.readyState !== 1) {
        setTimeout(() => {
            waitForWebsocket(websocket, retries - 1, success, fail)
        }, 100)
    } else {
        success()
    }
}

export default {
    createLocalSession,
    createNetworkSession,
    joinNetworkSession,
    watchNetworkSession,
    createEventWebsocket
}
