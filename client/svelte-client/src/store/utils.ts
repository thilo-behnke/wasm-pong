import {readable} from "svelte/store";

export const timer = ms => readable(ms, function(set) {
    setTimeout(() => {
        set(0)
    }, ms)
})
