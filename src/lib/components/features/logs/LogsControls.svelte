<script lang="ts">
    import { IconLetterD, IconLetterE, IconLetterI, IconLetterW} from "@tabler/icons-svelte";
    import tinycolor from "tinycolor2";

    import { logLevelDisplay, themeManager } from "../../../state.ts";
    import IconButton from "../../buttons/IconButton.svelte";

    const accentColorBright = "--accent-color-brighter";
    const backgroundMidColor = "--background-mid";
    const textDimColor = "--text-dim";

    let activeColor = backgroundMidColor;
    let inactiveColor = textDimColor;

    $: if ($themeManager.theme === "light") {
        activeColor = accentColorBright;
        inactiveColor = accentColorBright;
    }
</script>

<div class="LogsControls">
    <IconButton
        icon={IconLetterE}
        size={8}
        variant={$logLevelDisplay.error ? "filled" : "outline"}
        color={$logLevelDisplay.error ? activeColor : inactiveColor}
        on:click={() => logLevelDisplay.update((current) => ({...current, error: !$logLevelDisplay.error}))}
    />

    <IconButton
        icon={IconLetterW}
        size={8}
        variant={$logLevelDisplay.warn ? "filled" : "outline"}
        color={$logLevelDisplay.warn ? activeColor : inactiveColor}
        on:click={() => logLevelDisplay.update((current) => ({...current, warn: !$logLevelDisplay.warn}))}
    />

    <IconButton
        icon={IconLetterI}
        size={8}
        variant={$logLevelDisplay.info ? "filled" : "outline"}
        color={$logLevelDisplay.info ? activeColor : inactiveColor}
        on:click={() => logLevelDisplay.update((current) => ({...current, info: !$logLevelDisplay.info}))}
    />

    <IconButton
        icon={IconLetterD}
        size={8}
        variant={$logLevelDisplay.debug ? "filled" : "outline"}
        color={$logLevelDisplay.debug ? activeColor : inactiveColor}
        on:click={() => logLevelDisplay.update((current) => ({...current, debug: !$logLevelDisplay.debug}))}
    />
</div>

<style>
    .LogsControls {
        display: flex;
        gap: 3px;
        padding-top: 3px;
    }
</style>
