import {writable} from "svelte/store";

export type NetworkStore = {
    loading: boolean
}

const initialValue = () => ({
    loading: false
});

export const network = writable(initialValue())

export const networkContext = Symbol();
