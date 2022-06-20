<script lang="ts">
    import {localSessionInputs, Session} from "./game/session";

    export let session: Session;
</script>

<slot inputs={$localSessionInputs}></slot>
