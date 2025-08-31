<script lang="ts">
    import {
        isBufferingAudio,
        isConnected,
        isConnecting,
        isEstablishingConnectionState,
        isPlaying,
        streamerDisplay
    } from "../../../state.ts";
    import Art from "../../dataDisplay/Art.svelte";

    $: haveTextDetails = $streamerDisplay?.line1 || $streamerDisplay?.line2 || $streamerDisplay?.line3;
</script>

<div class="TrackInfo">
    <Art url={$streamerDisplay?.art_url} size={60} />

    <div class="details">
        {#if $isBufferingAudio}
            <span class="waiting-on-streamer">streamer is buffering audio</span>
        {:else if $isConnecting}
            <span class="waiting-on-streamer">streamer is connecting</span>
        {:else if haveTextDetails}
            <span class="details-line1">{$streamerDisplay?.line1 || ""}</span>
            <span class="details-line2">{$streamerDisplay?.line2 || ""}</span>
            <span class="details-line3">{$streamerDisplay?.line3 || ""}</span>
        {:else if !$isConnected && !$isEstablishingConnectionState}
            <span class="details-line1">No track details</span>
            <span class="details-line2">Not connected to a StreamMagic streamer</span>
        {:else if !$isPlaying}
            <div class="delayed-display">
                <span class="details-line1 dim">Not playing</span>
            </div>
        {:else}
            <div class="delayed-display" style="display: flex; flex-direction: column">
                <span class="details-line1 dim">No track details</span>
            </div>
        {/if}
    </div>
</div>

<style>
    :root {
        --art-size: 60px;
    }

    .TrackInfo {
        display: flex;
        gap: 0.7em;
        align-items: flex-start;
        max-width: 100%;
        padding: 3px;
        overflow: hidden;
        white-space: nowrap;
    }

    @keyframes oscillate {
        0%, 100% {
            opacity: 1.0;
        }
        50% {
            opacity: 0.65;
        }
    }

    .waiting-on-streamer {
        margin-top: 0.4em;
        font-weight: 500;
        font-size: 0.95em;
        color: var(--text-dim);

        animation: oscillate 3s ease-in-out 0s infinite;
    }

    .details {
        display: flex;
        flex-direction: column;
        overflow: hidden;
        padding-right: 0.3em;
        white-space: nowrap;

        & span {
            text-align: left;
            line-height: 1.30;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }

        & .details-line1 {
            font-size: 1.1em;
            margin-top: 0.4em;
            font-weight: 600;
        }

        & .details-line2 {
            font-size: 0.85em;

            @media (prefers-color-scheme: light) {
                font-weight: 500;
            }
        }

        & .details-line3 {
            font-size: 0.85em;
            color: var(--text-dim);
        }
    }

    @keyframes delayedDisplayAnimation {
        0% {
            opacity: 0;
        }
        99% {
            opacity: 0;
        }
        100% {
            opacity: 1;
        }
    }

    .delayed-display {
        animation: delayedDisplayAnimation 3s forwards;
    }

    .dim {
        color: var(--text-dim);
    }
</style>