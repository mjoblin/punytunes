<script lang="ts">
    import { IconQuestionMark } from "@tabler/icons-svelte";
    import tinycolor from "tinycolor2";

    import { themeManager } from "../../state.ts";

    type Variant = "filled" | "outline" | "subtle";

    // TODO: Investigate using CSS's "filter: brightness(1.2);" for hover
    // TODO: Improve display of disabled state (currently only "subtle" is explicitly handled)

    export var icon = IconQuestionMark;
    export var variant: Variant = "subtle";
    export var disabled: boolean = false;
    export var size: number = 20;
    export var fontSize: number = 12;
    export var color: string | undefined = undefined;
    export var borderRadius: string = "4px";
    export var stroke: number = 2;
    export var padding: string = "0.3em 0.3em";

    $: haveChild = !!$$slots.default;
    $: baseColor = color;

    let textDim: string;
    let textMax: string;
    let accentColor: string;
    let backgroundBrighter: string;
    let backgroundDim: string;

    $: {
        textDim = $themeManager.getCssVar("--text-dim");
        textMax = $themeManager.theme === "light" ? "white" : $themeManager.getCssVar("--text-max");
        accentColor = $themeManager.getCssVar("--accent-color");
        backgroundBrighter = $themeManager.getCssVar("--background-brighter");
        backgroundDim = $themeManager.getCssVar("--background-dim");
    }

    // Establish core color (and its dimmed flavor). This core color will be used for the button
    // background and border, in different ways depending on variant.
    $: {
        if (typeof color === "undefined") {
            baseColor = variant === "subtle" ? textDim : accentColor;
        } else if (color.startsWith("--")) {
            baseColor = $themeManager.getCssVar(color);
        }
    }

    $: colorDim = tinycolor(baseColor).darken(20).toString();

    // Each button variant might have a different color for its icon, background, and border.
    $: iconColor = baseColor;
    $: backgroundColor = baseColor;
    $: backgroundColorHover = baseColor;
    $: borderColor = baseColor;
    $: borderColorHover = baseColor;

    $: if (variant === "filled") {
        iconColor = disabled ? textDim : textMax;
        backgroundColor = baseColor;
        const backgroundLightened = tinycolor(backgroundColor).lighten(5).toString();
        backgroundColorHover = backgroundLightened;
        borderColor = baseColor;
        borderColorHover = backgroundLightened;
    } else if (variant === "outline") {
        iconColor = baseColor;
        backgroundColor = "transparent";
        backgroundColorHover = "transparent";
        const borderForTheme = $themeManager.theme === "light"
            ? baseColor
            : tinycolor(baseColor).darken(30).toString();
        borderColor = borderForTheme;
        borderColorHover = borderForTheme;
    } else if (variant === "subtle") {
        iconColor = disabled
            ? $themeManager.theme === "light"
                ? backgroundBrighter
                : colorDim
            : baseColor;
        backgroundColor = "transparent";
        backgroundColorHover = backgroundDim;
        borderColor = "transparent";
        borderColorHover = "transparent";
    }

    $: cssVarStyles =
        `--color:${baseColor};` +
        `--color-dim:${colorDim};` +
        `--icon-color:${iconColor};` +
        `--background-color:${backgroundColor};` +
        `--background-color-hover:${backgroundColorHover};` +
        `--border:1px solid ${borderColor};` +
        `--border-hover:1px solid ${borderColorHover};` +
        `--padding:${padding};` +
        `--content-font-size:${fontSize}px;` +
        `--border-radius:${borderRadius};`;
</script>

<button type="button" style={cssVarStyles} {disabled} on:click>
    <div class="button-content">
        <svelte:component this={icon} {size} {stroke} />
        {#if haveChild}
            <div class="button-content-child">
                <slot />
            </div>
        {/if}
    </div>
</button>

<style>
    button {
        color: var(--color);
        background-color: var(--background-color);
        box-sizing: border-box;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        border: var(--border);
        border-radius: var(--border-radius);
        font-family: inherit;
        font-size: 1em;
        padding: var(--padding);
        cursor: pointer;
        text-align: center;
        line-height: 1.1;

        transition: 120ms all ease-in-out;

        &:hover:enabled {
            background-color: var(--background-color-hover);
            border: var(--border-hover);
        }

        &:disabled {
            color: var(--color-dim);
            cursor: not-allowed;
        }
    }

    .button-content {
        display: flex;
        align-items: center;
        gap: 5px;
        color: var(--icon-color);
    }

    .button-content-child {
        font-size: var(--content-font-size);
    }
</style>