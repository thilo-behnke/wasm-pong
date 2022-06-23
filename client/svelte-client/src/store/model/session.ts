export enum SessionState {
    PENDING = 'PENDING', RUNNING = 'RUNNING', CLOSED = 'CLOSED'
}

export enum SessionType {
    LOCAL = 'LOCAL', HOST = 'HOST', PEER = 'PEER', OBSERVER = 'OBSERVER'
}

export type Player = {
    id: string,
    nr: number
}

export type Observer = {
    id: string
}

export type LocalSession = {
    session_id: string,
    state: SessionState,
    type: SessionType.LOCAL
}

export type NetworkSession = {
    session_id: string,
    type: SessionType.HOST | SessionType.PEER | SessionType.OBSERVER,
    state: SessionState,
    players: Player[],
    you: Player
}
export type Session = LocalSession | NetworkSession;

export function isNetworkSession(session: Session): session is NetworkSession {
    return !isLocalSession(session)
}

export function isLocalSession(session: Session): session is LocalSession {
    return !!session.type && session.type === SessionType.LOCAL
}
