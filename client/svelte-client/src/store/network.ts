import {writable} from "svelte/store";

export type NetworkStore = {
    loading: boolean,
    error?: {
        value: string,
        at: number
    }
}

const initialValue = () => ({
    loading: false,
});

export const network = writable<NetworkStore>(initialValue())
