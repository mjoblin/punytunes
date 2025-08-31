<script lang="ts">
    import { isHandlingAmplifier, isCbusAmpModeEnabled, isPreAmpModeEnabled, showVolumeControls } from "../state.ts";
    import Playhead from "./features/nowPlaying/Playhead.svelte";
    import StatusLine from "./features/nowPlaying/StatusLine.svelte";
    import TrackInfo from "./features/nowPlaying/TrackInfo.svelte";
    import TransportControls from "./features/nowPlaying/TransportControls.svelte";
    import VolumeControls from "./features/nowPlaying/VolumeControls.svelte";
</script>

<div class="Main">
    <div class="now-playing">
        <TrackInfo />
        {#if ($isHandlingAmplifier || $isPreAmpModeEnabled || $isCbusAmpModeEnabled) && $showVolumeControls}
            <VolumeControls />
        {/if}
    </div>
    <div class="playback-controls">
        <TransportControls />
        <Playhead />
    </div>
    <StatusLine />
</div>

<style>
    .Main {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .now-playing {
        display: flex;
        width: 100%;
        justify-content: space-between;
        align-items: flex-start;
    }

    .playback-controls {
        display: flex;
        gap: 10px;
        width: 100%;
        align-content: center;
    }
</style>
