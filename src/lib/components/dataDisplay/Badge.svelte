<script lang="ts">
    import tinycolor from "tinycolor2";

    import { themeManager } from "../../state.ts";

    export let color: string;

    let backgroundColor: string;
    let foregroundColor: string;

    $: {
        color = color ?? "--accent-color";
        backgroundColor = color?.startsWith("--") ? $themeManager.getCssVar(color) || "gray" : "gray";

        // Have the foreground be light or dark based on background luminance
        let badgeLuminance = tinycolor(backgroundColor).getLuminance();

        foregroundColor = $themeManager.theme === "light" ?
            "#ffffff" :
            badgeLuminance < 0.3 ? $themeManager.getCssVar("--text-max") : $themeManager.getCssVar("--text-min");
    }

    $: cssVarStyles = `--color:${backgroundColor};--text-color:${foregroundColor}`;
</script>

<div class="Badge" style={cssVarStyles}>
    <div class="content">
        <slot />
    </div>
</div>

<style>
    .Badge {
        height: 2em;
        padding: 0 1em;
        font-size: 0.75em;
        font-weight: 500;
        background-color: var(--color);
        border-radius: 100vh; /* Large border-radius forces 50% of container height */
        text-transform: uppercase;
        justify-content: center;
        align-items: center;
        line-height: 2.1em;
        color: var(--text-color);
        overflow: hidden;
        white-space: nowrap;

        @media (prefers-color-scheme: light) {
            font-weight: 600;
        }
    }
</style>