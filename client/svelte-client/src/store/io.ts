import type {Readable} from "svelte/store";
import {readable} from "svelte/store";

export const keysPressed: Readable<string[]> = readable([], function(set) {
    let keys = [];

    const onKeydown = ({key}) => {
        if (keys.includes(key)) {
            return;
        }
        keys = [...keys, key];
        set(keys);
    }
    const onKeyup = ({key}) => {
        if (!keys.includes(key)) {
            return;
        }
        keys = keys.filter(k => k !== key);
        set(keys);
    }

    document.addEventListener('keydown', onKeydown);
    document.addEventListener('keyup', onKeyup);

    return () => {
        document.removeEventListener('keydown', onKeydown);
        document.removeEventListener('keyup', onKeyup);
    }
});
