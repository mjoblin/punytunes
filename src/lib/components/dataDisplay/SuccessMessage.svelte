<script lang="ts">
    import { IconCheck } from "@tabler/icons-svelte";

    import { themeManager } from "../../state.ts";

    $: successColor = $themeManager.getCssVar("--success-color");
</script>

<div class="SuccessMessage">
    <IconCheck size={16} color={successColor} />
    <slot />
</div>

<style>
    .SuccessMessage {
        display: flex;
        align-items: center;
        gap: 5px;
        margin-bottom: 2px;
        font-size: 0.9rem;
        font-weight: 500;
    }
</style>