import {derived, Writable, writable} from "svelte/store";
import {getContext, onMount} from "svelte";

export const engineCanvas = writable();
export const engineCtx = writable();
export const width = writable(800);
export const height = writable(600);
export const pixelRatio = writable(window.devicePixelRatio);

// A more convenient store for grabbing all game props
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
