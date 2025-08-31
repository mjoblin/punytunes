<script lang="ts">
    import { nowPlaying, streamerDisplay, systemSources } from "../../state.ts";
    import Badge from "./Badge.svelte";

    const sourceClassColor: Record<string, string> = {
        "digital.coax": "--space-cadet",
        "digital.toslink": "--wheat",
        "digital.usb": "--robin-egg-blue",
        "stream.media": "--cerulean",
        "stream.radio": "--goldenrod",
        "stream.service.airplay": "--moss-green",
        "stream.service.cast": "--burnt-umber",
        "stream.service.roon": "--rose-quartz",
        "stream.service.spotify": "--sea-green",
        "stream.service.tidal": "--tomato",
    }

    export let id: string | undefined = undefined;

    // Note: The "source" is the streamer's current input (AirPlay, local media, internet radio,
    // etc); and the "playback source" is the associated stream source (computer/phone name for
    // AirPlay, NAS name for local media, etc).

    $: currentSourceId = id || $nowPlaying?.source?.id;
    $: currentSource = $systemSources?.sources.find((source) => source.id === currentSourceId);
    $: currentSourceDescription = currentSource?.description_locale || currentSource?.description;

    $: playbackSource = $streamerDisplay?.playback_source;
    $: playbackSourceDisplay = playbackSource?.toLowerCase() === currentSourceDescription?.toLowerCase() ? "" : playbackSource;
</script>

{#if currentSourceDescription}
    <Badge color={currentSource?.class ? sourceClassColor[currentSource.class] : "gray"}>
        <div class="AudioSource">
            <span>{currentSourceDescription}</span>
            {#if playbackSourceDisplay}
                <span class="playback-source">{playbackSourceDisplay}</span>
            {/if}
        </div>
    </Badge>
{/if}

<style>
    .AudioSource {
        display: flex;
        gap: 0.6em;
        overflow: hidden;
        white-space: nowrap;
    }

    .playback-source {
        font-weight: bold;

        @media (prefers-color-scheme: light) {
            font-weight: 500;
        }
    }
</style>