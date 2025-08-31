<script lang="ts">
    import { IconCurrentLocation } from "@tabler/icons-svelte";

    import { presets, showDetailedPresets, themeManager } from "../../../state.ts";
    import pubSub from "../../../pubSub.ts";
    import IconButton from "../../buttons/IconButton.svelte";
    import Switch from "../../buttons/SimpleDetailedSwitch.svelte";
    import Tooltip from "../../overlays/Tooltip.svelte";

    $: currentPreset = $presets?.presets?.find((preset) => preset.is_playing);
</script>

<div class="PresetsControls">
    <Tooltip label="scroll to current">
        <IconButton
            disabled={!currentPreset}
            icon={IconCurrentLocation}
            size={10}
            color={
                currentPreset
                    ? $themeManager.theme === "light"
                        ? "--accent-color-brighter"
                        : "--text-normal"
                    : "--background-brighter"
            }
            on:click={() => pubSub.publish("ScrollToCurrentPresetItem")}
        />
    </Tooltip>

    <Switch
        selected={$showDetailedPresets ? "right" : "left"}
        onChange={(side) => showDetailedPresets.set(side === "right")}
    />
</div>

<style>
    .PresetsControls {
        display: flex;
        gap: 3px;
    }
</style>