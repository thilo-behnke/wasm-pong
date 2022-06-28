import {get, Readable, readable, Unsubscriber, writable} from "svelte/store";
import type {Subscriber} from "svelte/types/runtime/store";

export const timer = ms => readable(ms, function(set) {
    setTimeout(() => {
        set(0)
    }, ms)
})

export function combined<T>(store1: Readable<T[]>, store2: Readable<T[]>): Readable<T[]> {
    const val1 = writable<T[]>([]);
    const val2 = writable<T[]>([]);

    const {set, subscribe} = writable<T[]>();

    const unsub1 = store1.subscribe(val => {
        val1.set(val);
        set([...val, ...get(val2)]);
    })
    const unsub2 = store2.subscribe(val => {
        val2.set(val);
        set([...get(val1), ...val]);
    })

    const customSubscribe = (run: Subscriber<T[]>, invalidate): Unsubscriber => {
        const unsub = subscribe(run, invalidate);
        return () => {
            unsub();
            unsub1();
            unsub2();
        }
    }
    return {
        subscribe: customSubscribe
    }
}
