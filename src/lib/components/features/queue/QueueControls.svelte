<script lang="ts">
    import { IconArrowAutofitHeight, IconCurrentLocation } from "@tabler/icons-svelte";

    import pubSub from "../../../pubSub.ts";
    import { followCurrentQueueItem, themeManager, showDetailedQueue } from "../../../state.ts";
    import IconButton from "../../buttons/IconButton.svelte";
    import Switch from "../../buttons/SimpleDetailedSwitch.svelte";
    import Tooltip from "../../overlays/Tooltip.svelte";

    $: followCurrentButtonColor = $themeManager.theme === "light"
        ? "--accent-color-brighter"
        : $followCurrentQueueItem ? "--background-mid" : "--text-dim";
</script>

<div class="QueueControls">
    <Tooltip label="follow current">
        <IconButton
            icon={IconArrowAutofitHeight}
            size={10}
            variant={$followCurrentQueueItem ? "filled" : "outline"}
            color={followCurrentButtonColor}
            on:click={() => followCurrentQueueItem.set(!$followCurrentQueueItem)}
        />
    </Tooltip>

    <Tooltip label="scroll to current">
        <IconButton
            icon={IconCurrentLocation}
            size={10}
            color={$themeManager.theme === "light" ? "--accent-color-brighter" : "--text-normal"}
            on:click={() => pubSub.publish("ScrollToCurrentQueueItem")}
        />
    </Tooltip>

    <Switch
        selected={$showDetailedQueue ? "right" : "left"}
        onChange={(side) => showDetailedQueue.set(side === "right")}
    />
</div>

<style>
    .QueueControls {
        display: flex;
        gap: 3px;
    }
</style>
