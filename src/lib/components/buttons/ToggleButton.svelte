<script lang="ts">
    import tinycolor from "tinycolor2";

    import { themeManager } from "../../state.ts";
    import IconButton from "./IconButton.svelte";

    export let icon;
    export let disabled: boolean = false;
    export let isOn: boolean = false;
    export let size: number = 12;
    export let stroke: number = 2;

    $: onColor = $themeManager.theme === "light"
        ? "--accent-color-brighter"
        : tinycolor($themeManager.getCssVar("--accent-color-bright")).brighten(20).toString();
</script>

<IconButton {icon} {size} {stroke} {disabled} color={isOn ? onColor : undefined} on:click />
