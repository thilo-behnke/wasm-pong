import type {
    GameObject,
    GameState,
    HostSessionSnapshot,
    NetworkSession,
    PeerSessionSnapshot,
    Session,
    SessionSnapshot
} from "./session";
import type {Input} from "./input";

export type SessionEventPayload = {
    actor: { id: string },
    event_type: string,
    reason: string,
    session: Session
}

export type NetworkSessionEventPayload = {
    actor: { id: string },
    event_type: string,
    reason: string,
    session: NetworkSession
}

export type InputEventPayload = {
    session_id: string,
    inputs: Input[],
    player_id: string,
    ts: number,
}

export type StatusEventPayload = {
    session_id: string,
    state: GameState
}

export type SessionEvenWrapper = {
    topic: 'session',
    event: SessionEventPayload
}

export type InputEventWrapper = {
    topic: 'input',
    event: InputEventPayload
}

export type TickEventWrapper = {
    topic: 'tick',
    event: SessionSnapshot
}

export type MoveEventWrapper = {
    topic: 'move',
    event: GameObject
}

export type StatusEventWrapper = {
    topic: 'status',
    event: StatusEventPayload
}

export type GameEventWrapper = SessionEvenWrapper | InputEventWrapper | MoveEventWrapper | TickEventWrapper | StatusEventWrapper;

export const isSessionEvent = (event: GameEventWrapper): event is SessionEvenWrapper => {
    return event.topic === 'session';
}

export const isInputEvent = (event: GameEventWrapper): event is InputEventWrapper => {
    return event.topic === 'input';
}

export const isMoveEvent = (event: GameEventWrapper): event is MoveEventWrapper => {
    return event.topic === 'move';
}

export const isTickEvent = (event: GameEventWrapper): event is TickEventWrapper => {
    return event.topic === 'tick';
}

export const isStatusEvent = (event: GameEventWrapper): event is StatusEventWrapper => {
    return event.topic === 'status';
}
