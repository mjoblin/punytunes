<script lang="ts">
    import { streamerDisplay } from "../../state.ts";
</script>

{#if $streamerDisplay?.format}
    <div class="AudioFormat">
        {$streamerDisplay?.format}
    </div>
{/if}

<style>
    .AudioFormat {
        color: var(--text-dim);
        font-size: 0.95em;
        line-height: 1.1;
        overflow: hidden;
    }
</style>