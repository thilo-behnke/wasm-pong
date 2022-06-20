<script lang="ts">
    import {keyboardInputs, Session} from "./game/session";

    export let session: Session;
</script>

<slot inputs={$keyboardInputs}></slot>


