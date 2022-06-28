import {derived, readable} from "svelte/store";
import {keysPressed} from "./io";
import type {Input} from "./model/input";

export const playerKeyboardInputs = derived(keysPressed, $keysPressed => {
    return $keysPressed.map((key): Input => {
        switch (key.toLowerCase()) {
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
})

export const getPlayerKeyboardInputs = (player_nr: number) => readable<Input[]>([], set => {
    return playerKeyboardInputs.subscribe(inputs => {
        set(inputs.filter(({player}) => player === player_nr));
    })
})
