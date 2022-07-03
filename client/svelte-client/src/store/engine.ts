import {FieldWrapper} from "wasm-app";
import {derived, Readable, Writable, writable} from "svelte/store";
import {getContext, onMount} from "svelte";
import type {GameObject} from "./model/session";
import type {Input} from "./model/input";
import type {Subscriber} from "svelte/types/runtime/store";

export const engineCanvas = writable();
export const engineCtx = writable();
export const width = writable(800);
export const height = writable(600);
export const pixelRatio = writable(window.devicePixelRatio);

// A more convenient store for grabbing all store props
export const props = deriveObject({
    width,
    height,
    pixelRatio,
    engineCanvas,
    engineCtx
});

export const renderContext = Symbol();

// https://svelte.dev/repl/79f4f3e0296a403ea988f74d332a7a4a?version=3.12.1
export const renderable = (render) => {
    const api: any = getContext(renderContext);
    const element = {
        ready: false,
        mounted: false,
        render: null,
        setup: null
    };
    if (typeof render === 'function') element.render = render;
    else if (render) {
        if (render.render) element.render = render.render;
        if (render.setup) element.setup = render.setup;
    }
    api.add(element);
    onMount(() => {
        element.mounted = true;
        return () => {
            api.remove(element);
            element.mounted = false;
        };
    });
}

function deriveObject (obj) {
    const keys = Object.keys(obj);
    const list = keys.map(key => {
        return obj[key];
    });
    return derived(list, (array) => {
        return array.reduce((dict, value, i) => {
            dict[keys[i]] = value;
            return dict;
        }, {});
    });
}

export type GameScore = {
    player1: number,
    player2: number,
}

export type GameFieldState = {
    ts: number,
    objects: GameObject[],
    score: GameScore
}

function createGameFieldStore(): Readable<GameFieldState> & {tick: (inputs: Input[], dt: number) => void, update: (objects: GameObject[]) => void} {
    const {subscribe, set} = writable<GameFieldState>({ts: 0, objects: [], score: {player1: 0, player2: 0}});

    const field = FieldWrapper.new();

    function tick(inputs: Input[], dt: number) {
        field.tick(inputs, dt);

        const objects = JSON.parse(field.objects());
        const ts = Date.now();
        const score = JSON.parse(field.game_state()) as {goals_player_1: number, goals_player_2: number};
        set({objects, ts, score: {player1: score.goals_player_1, player2: score.goals_player_2}});
    }

    function update(objects: GameObject[]) {
        set({objects, ts: Date.now(), score: {player1: 0, player2: 0}})
    }

    return {
        subscribe,
        tick,
        update
    }
}

export const gameField = createGameFieldStore();
