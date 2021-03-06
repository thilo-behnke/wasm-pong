import type {LocalSession, NetworkSession} from "../store/model/session";
import {SessionState, SessionType} from "../store/model/session";
import type {NetworkSessionEventPayload} from "../store/model/event";

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

async function createNetworkSession(): Promise<NetworkSession> {
    return fetch("/pong/api/create_session", {method: 'POST', headers: [['Content-Type', 'application/json']]})
        .then(sessionResponseHandler)
        .then(session => ({...session, type: SessionType.HOST}) as NetworkSession)
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

function createJoinLink(sessionId: string): string {
    return `${window.location.origin}${window.location.pathname}?join=${sessionId}`;
}

function createWatchLink(sessionId: string): string {
    return `${window.location.origin}${window.location.pathname}?watch=${sessionId}`;
}

async function joinNetworkSession(sessionId): Promise<NetworkSession> {
    return fetch("/pong/api/join_session", {
        method: 'POST',
        body: JSON.stringify({session_id: sessionId}),
        headers: [['Content-Type', 'application/json']]
    })
        .then(sessionResponseHandler)
        .then(session => ({...session, type: SessionType.PEER}) as NetworkSession)
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function watchNetworkSession(sessionId): Promise<NetworkSession> {
    return fetch("/pong/api/watch_session", {
        method: 'POST',
        body: JSON.stringify({session_id: sessionId}),
        headers: [['Content-Type', 'application/json']]
    })
        .then(sessionResponseHandler)
        .then(session => ({...session, type: SessionType.OBSERVER} as NetworkSession))
        .catch(err => {
            console.error(`Failed to create session: ${err}`);
            throw(err);
        });
}

async function sessionResponseHandler(response: Response): Promise<NetworkSession> {
    if (!response.ok) {
        return response.text().then(text => {
            return Promise.reject(`${response.status}: ${text}`)
        });
    }
    return response.json().then(({data}) => {
        console.debug(`session action result: ${JSON.stringify(data)}`)
        return data;
    }).then((event: NetworkSessionEventPayload) => ({you: event.actor, ...event.session}));
}

async function createEventWebsocket(session: NetworkSession): Promise<WebSocket> {
    console.debug("creating ws for session: ", session)
    const url = `/pong/ws?session_id=${session.session_id}&actor_id=${session.you.id}&connection_type=${session.type.toLowerCase()}`;
    return createWebsocket(url);
}

async function createWebsocket(path: string): Promise<WebSocket> {
    return new Promise((res, rej) => {
        const baseUrl = location.host.split(':')[0];
        const websocket = new WebSocket(`wss://${baseUrl}/${path}`);
        console.debug("ws initialized, not yet ready: ", websocket)
        waitForWebsocket(websocket, 10, () => {
            return res(websocket)
        }, () => rej())
    })
}

const waitForWebsocket = (websocket, retries, success, fail) => {
    if (retries <= 0) {
        console.error("ws not established successfully")
        return
    }
    if (websocket.readyState !== 1) {
        console.debug("ws not yet ready, sleep and check again in 100ms")
        setTimeout(() => {
            waitForWebsocket(websocket, retries - 1, success, fail)
        }, 100)
    } else {
        console.debug("ws ready!")
        success()
    }
}

export default {
    createLocalSession,
    createNetworkSession,
    joinNetworkSession,
    watchNetworkSession,
    createEventWebsocket,
    createJoinLink,
    createWatchLink
}
