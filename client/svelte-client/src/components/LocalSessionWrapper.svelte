<script lang="ts">
    import {localSessionInputs, Session} from "../store/session";

    export let session: Session;
</script>

<slot inputs={$localSessionInputs}></slot>
