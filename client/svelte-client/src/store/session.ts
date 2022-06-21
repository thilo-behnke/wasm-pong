import {get, Readable, readable, writable, derived} from "svelte/store";
import {keysPressed} from "./io";

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
    observers: Observer[]
}

export type Session = LocalSession | NetworkSession;

export type Input = {
    input: 'UP' | 'DOWN',
    obj_id: number,
    player: number
}

export type InputEventPayload = {
    session_id: string,
    inputs: Input[],
    player_id: string,
    player_nr: number,
    ts: number,
}

const player1KeyboardInputs = derived(
    keysPressed,
    $keysPressed => {
        return $keysPressed.map((key): Input => {
            switch(key.toLowerCase()) {
                case 'w':
                    return {input: 'UP', obj_id: 0, player: 1};
                case 's':
                    return {input: 'DOWN', obj_id: 0, player: 1}
                default:
                    return null
            }
        }).filter(it => !!it);
    }
)

const player2KeyboardInputs = derived(
    keysPressed,
    $keysPressed => {
        return $keysPressed.map((key): Input => {
            switch(key.toLowerCase()) {
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

const sessionEvents = readable([], function(set) {
    // TODO: Setup ws

    setInterval(() => {
        set([])
    }, 10)

    // TODO: Destroy ws
    return () => {}
})

const inputEvents = derived(sessionEvents, ([$sessionEvents]) => $sessionEvents.filter(({input}) => input === 'topic'));

const player1InputEvents = derived(inputEvents, ([$inputEvents]) => {
    return $inputEvents.filter(({player_nr}) => player_nr === 1)
})

const player2InputEvents = derived(inputEvents, ([$inputEvents]) => {
    return $inputEvents.filter(({player_nr}) => player_nr === 2)
})

export const localSessionInputs = derived([player1KeyboardInputs, player2KeyboardInputs], ([$player1KeyboardInputs, $player2KeyboardInputs]) => [...$player1KeyboardInputs, ...$player2KeyboardInputs])
export const hostNetworkSessionInputs = derived([player1KeyboardInputs, player2InputEvents], ([player1, player2]) => [...player1, ...player2])
export const peerNetworkSessionInputs = derived([player1InputEvents, player2KeyboardInputs], ([player1, player2]) => [...player1, ...player2])
export const observerNetworkSessionInputs = derived([player1InputEvents, player2InputEvents], ([player1, player2]) => [...player1, ...player2])

export const sessionStore = writable<Session>(null);
