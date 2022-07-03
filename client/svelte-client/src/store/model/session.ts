import type {Input} from "./input";

export enum SessionState {
    PENDING = 'PENDING', RUNNING = 'RUNNING', CLOSED = 'CLOSED'
}

export enum SessionType {
    LOCAL = 'LOCAL', HOST = 'HOST', PEER = 'PEER', OBSERVER = 'OBSERVER'
}

export type Actor = {
    id: string,
}

export type Player = Actor & {
    nr: number
}

export type GameObject = {
    id: string,
    orientation_x: number,
    orientation_y: number,
    shape_param_1: number,
    shape_param_2: number,
    vel_x: number,
    vel_y: number,
    x: number,
    y: number,
}

export type Observer = Actor

export const isPlayer = (actor: Actor): actor is Player => {
    return !!(actor as Player).nr
}

export const isObserver = (actor: Actor): actor is Observer => {
    return !isPlayer(actor);
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
    you: Actor
}

export type Session = LocalSession | NetworkSession;

export function isNetworkSession(session: Session): session is NetworkSession {
    return !isLocalSession(session)
}

export function isLocalSession(session: Session): session is LocalSession {
    return !!session.type && session.type === SessionType.LOCAL
}

export type HostSessionSnapshot = {
    session_id: string,
    inputs: Input[],
    objects: GameObject[],
    player_id: string,
    ts: number
}

export type PeerSessionSnapshot = {
    session_id: string,
    inputs: Input[],
    player_id: string,
    ts: number
}

export type SessionSnapshot = HostSessionSnapshot | PeerSessionSnapshot;

export type Heartbeat = {
    session_id: string,
    player_id: string,
    ts: number
}

export enum MessageType {
    Snapshot = "SessionSnapshot", Heartbeat = "HeartBeat"
}

export type Message = {
    msg_type: MessageType.Snapshot,
    payload: SessionSnapshot
} | {
    msg_type: MessageType.Heartbeat,
    payload: Heartbeat
}
