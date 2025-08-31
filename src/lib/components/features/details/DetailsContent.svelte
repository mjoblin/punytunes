<script lang="ts">
    import { afterUpdate, onDestroy } from "svelte";
    import { get } from "svelte/store";

    import { activeDetailsView, currentInfoView, detailsScrollPositions } from "../../../state.ts";
    import pubSub from "../../../pubSub.ts";
    import Dev from "../../detailViews/Dev.svelte";
    import Info from "../../detailViews/Info.svelte";
    import Logs from "../../detailViews/Logs.svelte";
    import Payloads from "../../detailViews/Payloads.svelte";
    import Presets from "../../detailViews/Presets.svelte";
    import Queue from "../../detailViews/Queue.svelte";
    import Sources from "../../detailViews/Sources.svelte";
    import Streamer from "../../detailViews/Streamer.svelte";
    import UserSettings from "../../detailViews/UserSettings.svelte";

    // Unsubscribe functions to be invoked on destroy.
    let unsubscribeFns: (() => void)[] = [];

    onDestroy(() => {
        unsubscribeFns.forEach((unsubscribeFn) => {
            unsubscribeFn();
        });
    });

    // Restore scroll position when a new details view is displayed
    let detailsContentDOMElement: HTMLDivElement;
    let restoreScrollPosition = false;

    afterUpdate(() => {
        if (restoreScrollPosition && detailsContentDOMElement && $activeDetailsView) {
            detailsContentDOMElement.scrollTop = get(detailsScrollPositions)[$activeDetailsView];
            restoreScrollPosition = false;
        }

        pubSub.publish("DetailViewUpdated");
    });

    unsubscribeFns.push(
        activeDetailsView.subscribe(() => {
            restoreScrollPosition = true;
        })
    );
</script>

<!-- Note: Display of "logs" and "payloads" is enabled via keyboard shortcuts -->

{#if $activeDetailsView}
    <div
        class="DetailsContent"
        class:debug-content={["payloads", "logs"].includes($activeDetailsView)}
        bind:this={detailsContentDOMElement}
        on:scroll={() => {
            // Store (in app state) the current scroll position so it can be restored later
            const top = detailsContentDOMElement.scrollTop;

            detailsScrollPositions.update((positions) => (
                {
                    ...positions,
                    [$activeDetailsView]: top,
                }
            ));
        }}
    >
        {#if $activeDetailsView === "queue"}
            <Queue />
        {:else if $activeDetailsView === "presets"}
            <Presets />
        {:else if $activeDetailsView === "sources"}
            <Sources />
        {:else if $activeDetailsView === "streamer"}
            <Streamer />
        {:else if $activeDetailsView === "info"}
            {#if $currentInfoView === "userSettings"}
                <UserSettings />
            {:else}
                <Info />
            {/if}
        {:else if $activeDetailsView === "payloads"}
            <Payloads />
        {:else if $activeDetailsView === "logs"}
            <Logs />
        {:else if $activeDetailsView === "dev"}
            <Dev />
        {/if}
    </div>
{/if}

<style>
    .DetailsContent {
        border: 1px solid #414141;
        border-radius: 3px;
        max-height: 350px;
        font-size: 0.9em;
        padding: 5px;
        background-color: var(--background-dimmer);
        line-height: 1.3;
        box-shadow: inset 0 3px 5px 0 rgba(0, 0, 0, 0.1),
        inset 0 -5px 10px 0 rgba(0, 0, 0, 0.1);
        overflow-y: auto;

        @media (prefers-color-scheme: light) {
            border: 1px solid var(--background-brighter);
            box-shadow: inset 0 0.5px 3px 0 rgba(0, 0, 0, 0.05),
            inset 0 -0.5px 5px 0 rgba(0, 0, 0, 0.05);
        }

        &.debug-content {
            background-color: var(--app-background-color);
        }
    }
</style>