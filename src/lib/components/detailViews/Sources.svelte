<script lang="ts">
    import { setSourceId } from "../../commands.ts";
    import { isConnected, isEstablishingConnectionState, systemSources, zoneState } from "../../state.ts";

    $: sources = $systemSources?.sources.toSorted((a, b) => a.preferred_order < b.preferred_order ? -1 : 1) ?? [];
    $: activeSourceId = $zoneState?.source;
</script>

<div class="Sources">
    {#if !$isConnected && !$isEstablishingConnectionState}
        <div class="detail-data-empty">Not connected to a StreamMagic streamer</div>
    {:else if sources.length <= 0}
        <div class="detail-data-empty">No sources</div>
    {:else}
        <table>
            {#each sources as source}
                <tr on:click={() => setSourceId(source.id)}>
                    <td class:active={source.id === activeSourceId}>{source.name}</td>
                    <td class:active={source.id === activeSourceId}
                        class="description">{source.description !== source.name ? source.description : ""}</td>
                </tr>
            {/each}
        </table>
    {/if}
</div>

<style>
    .Sources {
    }

    .description {
        color: var(--text-dim);
    }

    .active {
        color: var(--text-min);
        font-weight: 500;
    }
</style>
