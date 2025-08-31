<script lang="ts">
    import {
        IconArrowsShuffle,
        IconPlayerPauseFilled,
        IconPlayerPlayFilled,
        IconPlayerStopFilled,
        IconPlayerTrackNextFilled,
        IconPlayerTrackPrevFilled,
        IconRepeat
    } from "@tabler/icons-svelte";

    import {
        nextTrack,
        pause,
        play,
        previousTrack,
        setRepeatState,
        setShuffleState,
        stop,
        togglePlayback
    } from "../../../commands.ts";
    import {
        activeControls,
        isPlaying,
        isRepeatEnabled,
        isShuffleEnabled,
        playState,
        themeManager
    } from "../../../state.ts";
    import IconButton from "../../buttons/IconButton.svelte";
    import ToggleButton from "../../buttons/ToggleButton.svelte";

    // TODO: Consider having this component make use of the transportManager.

    $: canPauseOrStop = $isPlaying && $activeControls.some(
        (activeControl) => ["play_pause", "pause", "stop"].includes(activeControl)
    );

    $: canPlayOrResume =
        !$isPlaying &&
        $activeControls.some((activeControl) =>
            ["play_pause", "play"].includes(activeControl)
        );

    $: canPause = $activeControls.includes("pause");
    $: canTogglePlayback = $activeControls.includes("play_pause");
</script>

<div class="TransportControls">
    <!-- Previous track -->
    <IconButton
        icon={IconPlayerTrackPrevFilled}
        disabled={!$activeControls?.includes("track_previous")}
        on:click={() => previousTrack()}
    />

    <!-- Play/pause/stop/toggle -->
    {#if $isPlaying}
        {#if canTogglePlayback || canPause}
            <IconButton
                icon={IconPlayerPauseFilled}
                disabled={!canPauseOrStop}
                size={28}
                on:click={() => canTogglePlayback ? togglePlayback() : pause()}
            />
        {:else}
            <IconButton
                icon={IconPlayerStopFilled}
                disabled={!canPauseOrStop}
                size={28}
                on:click={() => stop()}
            />
        {/if}
    {:else}
        <!-- TODO: Add check for playing a Preset Id -->
        <IconButton
            icon={IconPlayerPlayFilled}
            disabled={!canPlayOrResume}
            size={28}
            on:click={() => canTogglePlayback ? togglePlayback() : play()}
        />
    {/if}

    <!-- Next track -->
    <IconButton
        icon={IconPlayerTrackNextFilled}
        disabled={!$activeControls?.includes("track_next")}
        on:click={() => nextTrack()}
    />

    <!-- Repeat and Shuffle toggles -->
    <div class="toggles">
        <ToggleButton
            isOn={$isRepeatEnabled}
            icon={IconRepeat}
            stroke={$themeManager.theme === "light" && $isRepeatEnabled ? 2.5 : 2}
            disabled={!$activeControls.includes("toggle_repeat")}
            on:click={() => setRepeatState($playState?.mode_repeat === "all" ? "off" : "all")}
        />
        <ToggleButton
            isOn={$isShuffleEnabled}
            icon={IconArrowsShuffle}
            stroke={$themeManager.theme === "light" && $isShuffleEnabled ? 2.5 : 2}
            disabled={!$activeControls.includes("toggle_shuffle")}
            on:click={() => setShuffleState($playState?.mode_shuffle === "all" ? "off" : "all")}
        />
    </div>
</div>

<style>
    .TransportControls {
        display: flex;
        align-items: center;
    }

    .toggles {
        display: flex;
        flex-direction: column;
    }
</style>
