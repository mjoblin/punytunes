<script lang="ts">
    import { IconInfoCircle, IconPlaylist, IconPlug, IconRadio, IconRouter } from "@tabler/icons-svelte";

    import type { DetailsView } from "../../../state.ts";
    import { activeDetailsView, appUpdateState } from "../../../state.ts";
    import IconButton from "../../buttons/IconButton.svelte";
    import LogsControls from "../logs/LogsControls.svelte";
    import PresetsControls from "../presets/PresetsControls.svelte";
    import QueueControls from "../queue/QueueControls.svelte";
    import Tooltip from "../../overlays/Tooltip.svelte";

    $: windowWidth = 0;
    $: showButtonText = windowWidth > 400;
    $: isUpdateAvailable = $appUpdateState.isUpdateAvailable;

    const buttonColor = "--text-dim";
    const buttonHighlightedColor = "--attention-color";

    const handleDetailSelectorClick = (detailsView: DetailsView) => {
        $activeDetailsView = $activeDetailsView === detailsView ? undefined : detailsView;
    };
</script>

<svelte:window bind:innerWidth={windowWidth} />

<div class="DetailsSelector">
    <!--
        Note: The #if for deciding whether or not to show the button text based on window width is
            not ideal. It might be possible in svelte 5 to pass in an empty slot and have the
            IconButton not still think there's a child and render it as a little bit of unwanted
            whitespace.

            Reference: https://github.com/sveltejs/svelte/issues/5604
    -->
    {#if showButtonText}
        <div class="details-selector-options">
            <IconButton
                icon={IconPlaylist}
                variant={$activeDetailsView === "queue" ? "outline" : "subtle"} size={14}
                fontSize={11}
                color={buttonColor}
                on:click={() => handleDetailSelectorClick("queue")}
            >
                Queue
            </IconButton>

            <IconButton
                icon={IconRadio}
                variant={$activeDetailsView === "presets" ? "outline" : "subtle"}
                size={14}
                fontSize={11}
                color={buttonColor}
                on:click={() => handleDetailSelectorClick("presets")}
            >
                Presets
            </IconButton>

            <IconButton
                icon={IconPlug}
                variant={$activeDetailsView === "sources" ? "outline" : "subtle"}
                size={14}
                fontSize={11}
                color={buttonColor}
                on:click={() => handleDetailSelectorClick("sources")}
            >
                Source
            </IconButton>

            <IconButton
                icon={IconRouter}
                variant={$activeDetailsView === "streamer" ? "outline" : "subtle"}
                size={14}
                fontSize={11}
                color={buttonColor}
                on:click={() => handleDetailSelectorClick("streamer")}
            >
                Streamer
            </IconButton>

            <IconButton
                icon={IconInfoCircle}
                variant={
                    isUpdateAvailable
                        ? "outline"
                        : $activeDetailsView === "info"
                            ? "outline"
                            : "subtle"
                }
                size={14}
                color={isUpdateAvailable ? buttonHighlightedColor : buttonColor}
                on:click={() => handleDetailSelectorClick("info")}
            />
        </div>
    {:else}
        <div class="details-selector-options">
            <Tooltip label="queue" placement="top">
                <IconButton
                    icon={IconPlaylist}
                    variant={$activeDetailsView === "queue" ? "outline" : "subtle"} size={14}
                    color={buttonColor}
                    on:click={() => handleDetailSelectorClick("queue")}
                />
            </Tooltip>

            <Tooltip label="presets" placement="top">
                <IconButton
                    icon={IconRadio}
                    variant={$activeDetailsView === "presets" ? "outline" : "subtle"}
                    size={14}
                    color={buttonColor}
                    on:click={() => handleDetailSelectorClick("presets")}
                />
            </Tooltip>

            <Tooltip label="sources" placement="top">
                <IconButton
                    icon={IconPlug}
                    variant={$activeDetailsView === "sources" ? "outline" : "subtle"}
                    size={14}
                    color={buttonColor}
                    on:click={() => handleDetailSelectorClick("sources")}
                />
            </Tooltip>

            <Tooltip label="streamer" placement="top">
                <IconButton
                    icon={IconRouter}
                    variant={$activeDetailsView === "streamer" ? "outline" : "subtle"}
                    size={14}
                    color={buttonColor}
                    on:click={() => handleDetailSelectorClick("streamer")}
                />
            </Tooltip>

            <Tooltip label="PunyTunes information" placement="top">
                <IconButton
                    icon={IconInfoCircle}
                    variant={
                        isUpdateAvailable
                            ? "outline"
                            : $activeDetailsView === "info"
                                ? "outline"
                                : "subtle"
                    }
                    size={14}
                    color={isUpdateAvailable ? buttonHighlightedColor : buttonColor}
                    on:click={() => handleDetailSelectorClick("info")}
                />
            </Tooltip>
        </div>
    {/if}

    <div class="selected-detail-controls">
        {#if $activeDetailsView === "queue"}
            <QueueControls />
        {:else if $activeDetailsView === "presets"}
            <PresetsControls />
        {:else if $activeDetailsView === "logs"}
            <LogsControls />
        {/if}
    </div>
</div>

<style>
    .DetailsSelector {
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 24px;
    }

    .details-selector-options {
        display: flex;
        gap: 3px;
        align-items: center;
    }
</style>