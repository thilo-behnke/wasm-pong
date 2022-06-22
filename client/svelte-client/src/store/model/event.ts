import type {Input} from "../session";
import type {NetworkSession, Session} from "./session";

export type SessionEventPayload = {
    actor: {id: string},
    event_type: string,
    reason: string,
    session: Session
}

export type NetworkSessionEventPayload = {
    actor: {id: string},
    event_type: string,
    reason: string,
    session: NetworkSession
}

export type InputEventPayload = {
    session_id: string,
    inputs: Input[],
    player_id: string,
    player_nr: number,
    ts: number,
}

