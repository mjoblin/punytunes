import { derived, get, type Readable } from "svelte/store";

import {
    nextTrack as streamerNextTrack,
    pause,
    play,
    previousTrack as streamerPreviousTrack,
    seek,
    stop,
    togglePlayback as streamerTogglePlayback,
} from "./commands.ts";
import { activeControls, isPlaying, playState, position } from "./state.ts";

type TransportState = {
    canPause: boolean;
    canPauseOrStop: boolean;
    canPlayNext: boolean;
    canPlayOrResume: boolean;
    canPlayPrevious: boolean;
    canSeek: boolean;
    canTogglePlayback: boolean;
    currentTrackDuration: number | undefined;
    isPlaying: boolean;
};

/**
 * Manages transport related state and actions.
 *
 * Abstracts away some of the complexities of interacting with the transport.
 *
 * This is currently pretty incomplete, but implements a foundation which can be built on if/as
 * necessary. Will be most useful when more than one app component wishes to interact with the
 * transport.
 */
const transportManager = () => {
    const transportState: Readable<TransportState | undefined> = derived(
        [activeControls, isPlaying, playState],
        ([activeControls, isPlaying, playState]) => {
            return {
                canPause: activeControls.includes("pause"),
                canPauseOrStop:
                    isPlaying &&
                    activeControls.some((activeControl) =>
                        ["play_pause", "pause", "stop"].includes(activeControl),
                    ),
                canPlayNext: activeControls.includes("track_next"),
                canPlayOrResume:
                    !isPlaying &&
                    activeControls.some((activeControl) =>
                        ["play_pause", "play"].includes(activeControl),
                    ),
                canPlayPrevious: activeControls.includes("track_previous"),
                canSeek: activeControls.includes("seek"),
                canTogglePlayback: activeControls.includes("play_pause"),
                currentTrackDuration: playState?.metadata?.duration ?? undefined,
                isPlaying,
            };
        },
    );

    /**
     * Seek the currently-playing track to the given offset in seconds, relative to its current
     * playback position.
     */
    const seekToOffset = async (offset: number) => {
        const state = get(transportState);
        const duration = state?.currentTrackDuration;

        if (!state?.canSeek) {
            return;
        }

        // We intentionally don't store the current position in the main transportState because
        // it's constantly updating; so we retrieve it here only when needed.
        const currentPosition = get(position);

        if (typeof duration === "undefined" || typeof currentPosition === "undefined") {
            return;
        }

        if (offset >= 0) {
            // Forwards
            await seek(Math.min(currentPosition + offset, duration));
        } else {
            // Backwards
            await seek(Math.max(currentPosition + offset, 0));
        }
    };

    // --------------------------------------------------------------------------------------------
    // Exposed transport control functions
    // --------------------------------------------------------------------------------------------

    /**
     * Skip to the next track.
     */
    const nextTrack = async () => {
        const state = get(transportState);

        if (state?.canPlayNext) {
            await streamerNextTrack();
        }
    };

    /**
     * Skip to the previous track.
     *
     * Will actually move the playhead to the start of the current track, unless already at the
     * ery beginning of the current track (in which case the previous track will be played).
     */
    const previousTrack = async () => {
        const state = get(transportState);

        if (state?.canPlayPrevious) {
            await streamerPreviousTrack();
        }
    };

    /**
     * Seek backwards in the currently-playing track by the given number of seconds.
     */
    const seekBackwards = async (seconds: number = 10) => {
        await seekToOffset(-seconds);
    };

    /**
     * Seek forwards in the currently-playing track by the given number of seconds.
     */
    const seekForwards = async (seconds: number = 10) => {
        await seekToOffset(seconds);
    };

    /**
     * Toggle playback. This will attempt to use the streamer's "toggle playback" feature if
     * possible. If not possible, then it will attempt to either pause, play, or stop, depending
     * on current playback state.
     */
    const togglePlayback = async () => {
        const state = get(transportState);

        if (state?.isPlaying) {
            if (state?.canTogglePlayback) {
                await streamerTogglePlayback();
            } else if (state?.canPause) {
                await pause();
            } else {
                await stop();
            }
        } else {
            if (state?.canTogglePlayback) {
                await streamerTogglePlayback();
            } else {
                await play();
            }
        }
    };

    return {
        subscribe: transportState.subscribe,
        nextTrack,
        previousTrack,
        seekBackwards,
        seekForwards,
        togglePlayback,
    };
};

export default transportManager;
