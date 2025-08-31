<script lang="ts">
    import { IconListDetails, IconMenu2 } from "@tabler/icons-svelte";

    import { themeManager } from "../../state.ts";
    import IconButton from "./IconButton.svelte";
    import Tooltip from "../overlays/Tooltip.svelte";

    type SwitchOptions = "left" | "right";

    export var selected: SwitchOptions;
    export var size = 10;
    export var onChange = (activeSide: SwitchOptions): void => {
    };

    const accentColorBright = "--accent-color-brighter";
    const backgroundMidColor = "--background-mid";
    const textDimColor = "--text-dim";

    $: leftButtonColor = $themeManager.theme === "light"
        ? accentColorBright
        : selected === "left" ? backgroundMidColor : textDimColor;

    $: rightButtonColor = $themeManager.theme === "light"
        ? accentColorBright
        : selected === "right" ? backgroundMidColor : textDimColor;
</script>

<div class="Switch">
    <Tooltip label="simple view">
        <IconButton
            icon={IconMenu2}
            variant={selected === "left" ? "filled" : "outline"}
            {size}
            color={leftButtonColor}
            borderRadius="4px 0 0 4px"
            on:click={() => onChange("left")}
        />
    </Tooltip>

    <Tooltip label="detailed view">
        <IconButton
            icon={IconListDetails}
            variant={selected === "right" ? "filled" : "outline"}
            {size}
            color={rightButtonColor}
            borderRadius="0 4px 4px 0"
            on:click={() => onChange("right")}
        />
    </Tooltip>
</div>

<style>
    .Switch {
        display: flex;
        gap: 0;
    }
</style>