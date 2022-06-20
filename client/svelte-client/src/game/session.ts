import {get, Readable, readable, writable, derived} from "svelte/store";
import {keysPressed} from "./io";

export enum SessionState {
    PENDING = 'PENDING', RUNNING = 'RUNNING', CLOSED = 'CLOSED'
}

export enum SessionType {
    LOCAL = 'LOCAL', HOST = 'HOST', PEER = 'PEER', OBSERVER = 'OBSERVER'
}

export type Player = {
    id: string
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
    observers: Observer[]
}

export type Session = LocalSession | NetworkSession;

export type Input = {
    input: 'UP' | 'DOWN',
    obj_id: number,
    player: number
}

enum InputMethod {
    UNDEFINED, KEYBOARD, NETWORK
}

export const keyboardInputs = derived(
    keysPressed,
    $keysPressed => {
        return $keysPressed.map((key): Input => {
            switch(key.toLowerCase()) {
                case 'w':
                    return {input: 'UP', obj_id: 0, player: 1};
                case 's':
                    return {input: 'DOWN', obj_id: 0, player: 1}
                case 'arrowup':
                    return {input: 'UP', obj_id: 1, player: 2}
                case 'arrowdown':
                    return {input: 'DOWN', obj_id: 1, player: 2}
                default:
                    return null
            }
        }).filter(it => !!it);
    }
)

const inputMethod = writable(InputMethod.UNDEFINED)

function initialValue(): Session {
    return null
}

export const sessionStore = writable<Session>(null);
