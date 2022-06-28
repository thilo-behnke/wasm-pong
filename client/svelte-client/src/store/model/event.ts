import type {Input} from "../session";
import type {NetworkSession, Session} from "./session";

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

export type GameEvent = SessionEventPayload | NetworkSessionEventPayload | InputEventPayload;

export type SessionEvenWrapper = {
    topic: 'session',
    event: SessionEventPayload
}

export type InputEventWrapper = {
    topic: 'input',
    event: InputEventPayload
}

export type GameEventWrapper = SessionEvenWrapper | InputEventWrapper;

export const isSessionEvent = (event: GameEventWrapper): event is SessionEvenWrapper => {
    return event.topic == 'session';
}

export const isInputEvent = (event: GameEventWrapper): event is InputEventWrapper => {
    return event.topic == 'input';
}
