<script lang="ts">
    import { IconExclamationCircle } from "@tabler/icons-svelte";

    import { themeManager } from "../../state.ts";

    $: attentionColor = $themeManager.getCssVar("--attention-color");
</script>

<div class="WarningMessage">
    <IconExclamationCircle size={16} color={attentionColor} />
    <slot />
</div>

<style>
    .WarningMessage {
        display: flex;
        align-items: center;
        gap: 5px;
        margin-bottom: 2px;
        font-size: 0.9rem;
        font-weight: 500;
        color: var(--attention-color);
    }
</style>