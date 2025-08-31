<script lang="ts">
    import { afterUpdate, onDestroy } from "svelte";
    import { inview } from "svelte-inview";

    import type { PresetItem } from "../../../types/generated/streammagic_payloads/PresetItem.ts";
    import { playPresetId } from "../../commands.ts";
    import { isConnected, isEstablishingConnectionState, presets, showDetailedPresets } from "../../state.ts";
    import pubSub from "../../pubSub.ts";
    import Art from "../dataDisplay/Art.svelte";

    // Unsubscribe functions to be invoked on destroy.
    let unsubscribeFns: (() => void)[] = [];

    let imageSize = 25;
    let cssVarStyles = `--image-size:${imageSize}px;`;

    // Key: dom preset id; Value: is it in view
    let presetDomIdsInView: Record<string, Record<string, boolean>> = { data: {} };

    let firstPresetInView: PresetItem | undefined;
    let scrollToFirstPreset = false;

    const prestIdToDomId = (position: number | null) => `pos_id_${position}`;

    // --------------------------------------------------------------------------------------------
    // Lifecycle functions

    afterUpdate(() => {
        // The new view mode has finished rendering, so scroll to the preset that was at the top
        // before the view mode was changed
        if (scrollToFirstPreset && firstPresetInView) {
            document.getElementById(prestIdToDomId(firstPresetInView.id))?.scrollIntoView(
                { behavior: "smooth", block: "start" }
            );
        }

        scrollToFirstPreset = false;
        pubSub.publish("DetailViewUpdated");
    });

    onDestroy(() => {
        unsubscribeFns.forEach((unsubscribeFn) => {
            unsubscribeFn();
        });
    });

    // --------------------------------------------------------------------------------------------

    unsubscribeFns.push(
        showDetailedPresets.subscribe(() => {
            // The user has switched view mode, so find what preset is currently at the top
            firstPresetInView = $presets?.presets?.find(
                (preset) => !!presetDomIdsInView.data[prestIdToDomId(preset.id)]
            );
            scrollToFirstPreset = true;
        })
    );

    unsubscribeFns.push(
        pubSub.subscribe("ScrollToCurrentPresetItem", () => {
            const currentPreset = $presets?.presets?.find((preset) => preset.is_playing);

            currentPreset
            && typeof currentPreset.id === "number"
            && document.getElementById(prestIdToDomId(Math.max(currentPreset.id - 1, 1)))?.scrollIntoView(
                { behavior: "smooth", block: "start" }
            );
        })
    );

    // --------------------------------------------------------------------------------------------

    $: allPresets = $presets?.presets ?? [];
</script>

<div class="Presets" style={cssVarStyles}>
    {#if !$isConnected && !$isEstablishingConnectionState}
        <div class="detail-data-empty">Not connected to a StreamMagic streamer</div>
    {:else if allPresets.length <= 0}
        <div class="detail-data-empty">No presets</div>
    {:else}
        <table class="table">
            {#each allPresets as preset}
                <tr
                    id={prestIdToDomId(preset.id)}
                    class:detailed-view={showDetailedPresets}
                    on:click={() => preset.id && playPresetId(preset.id)}
                    use:inview={{ threshold: 0.5 }}
                    on:inview_change={(event) => {
                        const { inView, node } = event.detail;

                        // The "data" level of indirection is to prevent every set done here
                        // resulting in a svelte update (which in turn calls afterUpdate()).
                        const data = presetDomIdsInView.data;
                        data[node.id] = inView;
                    }}
                >
                    <td class:active={preset.is_playing} class="preset-id right-align">
                        {preset.id}
                    </td>

                    {#if $showDetailedPresets}
                        <td class="art" class:active={preset.is_playing}>
                            <Art url={preset.art_url} size={imageSize} />
                        </td>
                    {/if}

                    <td class:active={preset.is_playing}>
                        {preset.name}
                    </td>

                    <td class:active={preset.is_playing} class="preset-type right-align">
                        {preset.type}
                    </td>
                </tr>
            {/each}
        </table>
    {/if}
</div>

<!-- NOTE: Some table style are defined in main styles.css -->

<style>
    .Presets {
        overflow: hidden;
    }

    .preset-id {
        width: 15px;
        color: var(--text-dim);
    }

    .art {
        width: var(--image-size);
    }

    .preset-type {
        color: var(--text-dim);
    }

    tr.detailed-view {
        vertical-align: middle;
    }

    .right-align {
        text-align: right;
    }

    .active {
        color: var(--text-min);
        font-weight: 500;
    }
</style>
