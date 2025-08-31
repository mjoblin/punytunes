<script lang="ts">
    import { fade } from "svelte/transition";

    import { isActivating } from "../../state.ts";
</script>

<div class="ActivatingIndicator">
    {#if $isActivating}
        <div class="indicator" transition:fade={{ duration: 250 }} />
    {/if}
</div>

<style>
    .ActivatingIndicator {
        width: 3px;
        height: 3px;

        & .indicator {
            width: 3px;
            height: 3px;
            border-radius: 99px;
            background: #00afaf;
        }
    }
</style>