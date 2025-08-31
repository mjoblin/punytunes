<!--
    Main power button. Turns streamer on/off. If an amplifier is also being handled, then ensure
    the amplifier's power state matches the streamer power state.
-->

<script lang="ts">
    import { IconPower } from "@tabler/icons-svelte";

    import { isConnected, isHandlingAmplifier, isPowerOn, themeManager } from "../../state.ts";
    import { amplifierPowerOff, amplifierPowerOn, powerToggle } from "../../commands.ts";

    $: colors = $themeManager.theme === "light"
        ? {
            on: {
                color: "#2e9e2ece",
                background: "#2e9e2e20"
            },
            standby: {
                color: "#2e7e9ebe",
                background: "#2e7e9e1e"
            },
            notConnected: {
                color: $themeManager.getCssVar("--background-brighter"),
                background: $themeManager.getCssVar("--background-dim")
            }
        } : {
            on: {
                color: "#ceeecece",
                background: "#ceeece20"
            },
            standby: {
                color: "#aecfeebe",
                background: "#aecfee1e"
            },
            notConnected: {
                color: $themeManager.getCssVar("--background-brighter"),
                background: $themeManager.getCssVar("--background-dimmer")
            }
        };

    $: activeColors = $isPowerOn ? colors.on : $isConnected ? colors.standby : colors.notConnected;
    $: cssVarStyles = `--color:${activeColors.color};--background-color:${activeColors.background}`;
</script>

<button
    class="PowerButton"
    style={cssVarStyles}
    disabled={!$isConnected}
    on:click={() => {
        powerToggle();

        if ($isHandlingAmplifier) {
            if ($isPowerOn) {
                amplifierPowerOff();
            } else {
                amplifierPowerOn();
            }
        }
    }}
>
    <div class="power-button-icon">
        <IconPower color={activeColors.color} size={17} />
    </div>
</button>

<style>
    .PowerButton {
        width: 27px;
        height: 27px;
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