<!--
    Amplifier power button. Turns the amplifier on/off. This exists to allow for the amplifier to
    be turned on/off separately from the streamer (usually only necessary to get the two devices'
    power states back in sync when they drift). This power button is smaller and more subtle
    than the main <PowerButton>.
-->

<script lang="ts">
    import { IconPower } from "@tabler/icons-svelte";

    import { amplifierState, osType, themeManager } from "../../state.ts";
    import { amplifierPowerToggle } from "../../commands.ts";

    $: colors = $themeManager.theme === "light"
        ? {
            on: {
                color: "#1010104e",
                background: "#10101010"
            },
            standby: {
                color: "#1010102e",
                background: "#1010100e"
            },
        } : {
            on: {
                color: "#aeaeae7e",
                background: "#aeaeae20"
            },
            standby: {
                color: "#8e8e8e4e",
                background: "#8e8e8e1e"
            },
        };

    $: activeColors = $amplifierState?.is_powered_on ? colors.on : colors.standby;
    $: cssVarStyles = `--color:${activeColors.color};--background-color:${activeColors.background}`;
</script>

<button
    class="AmplifierPowerButton"
    style={cssVarStyles}
    on:click={amplifierPowerToggle}
>
    <div class="power-button-icon">
        <IconPower color={activeColors.color} size={$osType === "Windows_NT" ? 14 : 13} />
    </div>
</button>

<style>
    .AmplifierPowerButton {
        width: 20px;
        height: 20px;
        border-radius: 100px;
        box-sizing: border-box;
        display: inline-flex;
        align-items: center;
        justify-content: center;

        color: var(--color);
        background-color: var(--background-color);
        border: 1.5px solid var(--color);

        & .power-button-icon {
            display: flex;
            align-items: center;
            justify-content: center;
            text-align: center;
        }
    }
</style>