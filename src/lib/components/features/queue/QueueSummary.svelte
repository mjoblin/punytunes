<script lang="ts">
    import { activeSourceId, position, queueList } from "../../../state.ts";
    import { prettyDuration } from "../../../utils.ts";

    $: entries = $queueList?.items ?? [];

    // Length of the queue in seconds
    $: queueSecs = entries.reduce(
        (accumSecs, entry) => accumSecs + (entry.metadata?.duration ?? 0),
        0
    );

    // Already-played duration count
    $: completedProgressSecs = typeof $queueList?.play_postition === "number" ?
        entries
            .filter((entry) => typeof entry.position === "number" && entry.position < $queueList?.play_postition)
            .reduce((accumProgress, entry) => accumProgress + (entry.metadata?.duration ?? 0), 0)
        : 0;

    // Total progress through the queue, in seconds. Include the current position if playing
    // locally-streamed media.
    $: totalProgressSecs = completedProgressSecs + ($activeSourceId === "MEDIA_PLAYER" ? ($position ?? 0) : 0);
</script>

<div class="QueueSummary">
    <div>
        <span>{entries.length} track{entries.length > 0 ? "s" : ""},</span>
        <span>{prettyDuration(queueSecs, true, true)}</span>
    </div>
    <div>
        <span class="label">progress: </span>
        <span>{(queueSecs > 0 ? (totalProgressSecs / queueSecs) * 100 : 0).toFixed(1)}%</span>
    </div>
</div>

<style>
    .QueueSummary {
        display: flex;
        gap: 10px;
        color: var(--text-mid);
    }

    .label {
        color: var(--text-dim);
    }
</style>
