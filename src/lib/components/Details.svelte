<script lang="ts">
    import { activeDetailsView, detailsPosition } from "../state.ts";

    import DetailsContent from "./features/details/DetailsContent.svelte";
    import DetailsSelector from "./features/details/DetailsSelector.svelte";
</script>

<main
    class="Details"
    class:details-bottom-and-open={$detailsPosition === "bottom" && $activeDetailsView}
    class:details-bottom-and-closed={$detailsPosition === "bottom" && !$activeDetailsView}
    class:details-top-and-open={$detailsPosition === "top" && $activeDetailsView}
    class:details-top={$detailsPosition === "top"}
>
    {#if $detailsPosition === "top"}
        <DetailsContent />
        <DetailsSelector />
    {:else}
        <DetailsSelector />
        <DetailsContent />
    {/if}
</main>

<style>
    .Details {
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 7px;
    }

    .details-bottom-and-open {
        padding-bottom: 12px;
    }

    .details-bottom-and-closed {
        padding-bottom: 8px;
    }

    .details-top-and-open {
        padding-top: 4px;
    }
</style>