<script lang="ts">
    import { IconExclamationCircle } from "@tabler/icons-svelte";

    import { themeManager } from "../../state.ts";

    export let withIcon = true;

    $: attentionColor = $themeManager.getCssVar("--alert-color");
</script>

<div class="ErrorMessage">
    {#if withIcon}
        <IconExclamationCircle size={16} color={attentionColor} stroke={3} />
    {/if}
    <slot />
</div>

<style>
    .ErrorMessage {
        display: flex;
        align-items: center;
        gap: 5px;
        margin-bottom: 2px;
        font-size: 0.9rem;
        font-weight: 500;
        color: var(--alert-color);
    }
</style>