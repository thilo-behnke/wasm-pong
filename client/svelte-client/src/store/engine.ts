import {FieldWrapper} from "wasm-app";
import {derived, get, readable, Readable, Writable, writable} from "svelte/store";
import {getContext, onMount} from "svelte";
import type {GameObject, GameScore, GameState} from "./model/session";
import type {Input} from "./model/input";
import type {Subscriber} from "svelte/types/runtime/store";
import {subscribe} from "svelte/internal";
import Fps from "../components/Fps.svelte";

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

export type GameFieldState = {
    ts: number,
    objects: GameObject[],
    state: GameState,
    meta: {
        fps: number
    }
}

type FpsStore = Readable<number> & {
    inc: () => void;
}

const createFpsStore = (): FpsStore => {
    const fps = writable<number>();
    const frameCounter = writable<{fps: number, current: number, lastReset: number}>({fps: 0, current: 0, lastReset: 0});

    frameCounter.subscribe(({fps: updatedFps}) => {
        fps.set(updatedFps);
    })

    const inc = () => {
        const now = Date.now();
        frameCounter.update(prev => {
            const {current, lastReset} = prev;
            const diff = now - lastReset;
            if (diff > 1_000) {
                const fps = parseFloat((current / diff * 1_000).toFixed(2));
                return {fps, current: 0, lastReset: now };
            }
            return {...prev, current: current + 1};
        });
    }

    return {
        subscribe: fps.subscribe,
        inc
    }
}

export type GameFieldStore = Readable<GameFieldState> & {tick: (inputs: Input[], dt: number) => void, update: (objects: GameObject[], state: GameState) => void};

function createGameFieldStore(): GameFieldStore {
    const {subscribe, set} = writable<GameFieldState>({ts: 0, objects: [], state: {score: {player_1: 0, player_2: 0}}, meta: {fps: 0}});

    const fps = createFpsStore();

    const field = FieldWrapper.new();

    function tick(inputs: Input[], dt: number) {
        field.tick(inputs, dt);

        const objects = JSON.parse(field.objects());
        const ts = Date.now();
        const state = JSON.parse(field.game_state()) as GameState;

        fps.inc();

        const meta = {fps: get(fps)};
        set({objects, ts, state, meta} as GameFieldState);
    }

    function update(objects: GameObject[], state: GameState) {
        fps.inc();
        const meta = {fps: get(fps)};
        set({objects, ts: Date.now(), state, meta});
    }

    return {
        subscribe,
        tick,
        update
    }
}

export const gameField = createGameFieldStore();
