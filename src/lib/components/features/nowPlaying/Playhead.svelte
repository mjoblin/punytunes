<!--
    TODO: Can the clickable track area be made taller? Currently it's quite skinny (because we want
        to render it skinny), which makes it a little finicky to use: the user has to both click
        *and* release the mouse button within the skinny track for a seek to take effect.
-->

<script lang="ts">
    import { onDestroy } from "svelte";

    import { activeControls, playState, position, positionNormalized } from "../../../state.ts";
    import { seek } from "../../../commands.ts";
    import { prettyDuration } from "../../../utils.ts";

    const ONE_HOUR = 60 * 60;

    // Unsubscribe functions to be invoked on destroy.
    let unsubscribeFns: (() => void)[] = [];

    onDestroy(() => {
        unsubscribeFns.forEach((unsubscribeFn) => {
            unsubscribeFn();
        });
    });

    // Store the width of the span used to display the current track position. This width will be
    // determined based on rendering a string like "00:00" to an off-screen element. The goal is
    // to ensure the span has a min-width which is wide enough to contain the widest-displayed
    // text, to prevent subtle element shifts when displaying a thinner current value (such as
    // "01:11").
    let currentPositionWidth;

    // The application's $position might be undefined, which is a valid state representing
    // "I'm probably not playing/paused right now". We want to display this as "--:--", but we
    // also want to allow the slider to render at the zero position. So we distinguish between the
    // application's $position and this component's positionBindable ("bindable"
    // because it is guaranteed to be a number which can be bound to the slider's value).
    let playheadPositionBindable = $position || 0;

    // Manage changes to the playhead position ----------------------------------------------------
    //
    // When the user clicks (and potentially drags) on the playhead, we want to stop updating the
    // bound value on the input range. If we don't do that then the playhead position will move
    // based on $position updates while the user is also trying to move the playhead.
    //
    // When the user releases the playhead, we want to ignore the current $position value until
    // the next update comes in from the streamer. The next update from the streamer should reflect
    // the position that was just set (seeked to) by the user. If we don't continue to ignore
    // $position until the next updates comes in, then on mouse release the playhead will likely
    // move back to an old position before then quickly changing to the new updated position.
    //
    // Finally, we use "userSetPositionNormalized" to optimistically set the rendered progress
    // position to whatever the user set it to, until the next update comes it.

    let ignorePositionUpdates = false;
    let waitingForNextPositionUpdate = false;
    let userSetPositionNormalized = 0;

    $: if (!ignorePositionUpdates && !waitingForNextPositionUpdate) {
        playheadPositionBindable = $position || 0;
    }

    unsubscribeFns.push(
        position.subscribe(() => {
            if (waitingForNextPositionUpdate) {
                waitingForNextPositionUpdate = false;
                ignorePositionUpdates = false;
            }
        })
    );

    // --------------------------------------------------------------------------------------------

    $: canSeek = $activeControls.includes("seek");
    $: currentTrackDuration = $playState?.metadata?.duration;
    $: prettyTrackDuration = prettyDuration(currentTrackDuration);
    $: prettyTrackDurationZeros = prettyTrackDuration.replace(/[0-9]/g, "0");

    // Determine the display of the current time. If the total time is > 1hr then we force the
    // inclusion of hours in the current time.
    $: prettyTrackPosition = (currentTrackDuration || 0) < ONE_HOUR
        ? prettyDuration($position)
        : prettyDuration($position, false, false, true); // Force hours to be included

    $: cssVarStyles =
        `--progress:${
            waitingForNextPositionUpdate
                ? userSetPositionNormalized * 100
                : $positionNormalized
                    ? $positionNormalized * 100
                    : 0
        }%;`;

    // Handlers -----------------------------------------------------------------------------------

    const handlePlayheadSeek = async (e: MouseEvent) => {
        // Trigger a streamer position seek. This will (relatively quickly) be followed by a
        // streamer position update which should match where we're seeking to.
        const targetSecs = parseInt((e.target as HTMLInputElement).value);
        await seek(targetSecs);

        // Set the userSetPositionNormalized position to wherever was clicked. This will become the
        // rendered position until the next streamer position update arrives.
        const target = e.target as HTMLInputElement;
        const value = +target.value;
        const max = +target.max;

        if (max > 0) {
            userSetPositionNormalized = value / max;
        } else {
            userSetPositionNormalized = 0;
        }
    };

    const handlePlayheadMouseDown = () => {
        // When the playhead is clicked, start ignoring position updates coming in from the
        // streamer. This is to prevent the streamer's position updates from interfering with the
        // user's manual updates.
        ignorePositionUpdates = true;
    };

    const handlePlayheadMouseUp = () => {
        waitingForNextPositionUpdate = true;
    };
</script>

<div class="Playhead">
    <div class="off-screen" bind:clientWidth={currentPositionWidth}>
        <span class="time">{prettyTrackDurationZeros}</span>
    </div>

    <span style="min-width: {currentPositionWidth}px" class="time">{prettyTrackPosition}</span>

    <input
        style={cssVarStyles}
        class:can-seek={canSeek}
        disabled={!canSeek || typeof $position === "undefined"}
        type="range"
        min="0"
        max={currentTrackDuration}
        step="1"
        bind:value={playheadPositionBindable}
        on:mousedown={handlePlayheadMouseDown}
        on:mouseup={handlePlayheadMouseUp}
        on:click={handlePlayheadSeek}
    />

    <span class="time">{prettyTrackDuration}</span>
</div>

<style>
    .Playhead {
        flex-grow: 1;
        display: flex;
        align-items: center;
        gap: 3px;
    }

    .off-screen {
        position: absolute;
        left: -9999px;
        visibility: hidden;
    }

    .time {
        font-size: 0.7em;
        color: var(--text-mid);
        white-space: nowrap;
    }

    input[type="range"] {
        appearance: none;
        cursor: not-allowed;
        width: 100%;
        height: 3px;
        border: none;
        padding: 0;
        border-radius: 2px;
        background: linear-gradient(
            to right,
            #6c6f76 0%,
            #6c6f76 var(--progress),
            var(--background-mid) var(--progress),
            var(--background-mid) 100%
        );

        &.can-seek {
            background: linear-gradient(
                to right,
                var(--accent-color-bright) 0%,
                var(--accent-color-bright) var(--progress),
                var(--background-mid) var(--progress),
                var(--background-mid) 100%
            );
        }

        @media (prefers-color-scheme: light) {
            background: linear-gradient(
                to right,
                #6c6f76 0%,
                #6c6f76 var(--progress),
                var(--background-mid) var(--progress),
                var(--background-mid) 100%
            );

            &.can-seek {
                background: linear-gradient(
                    to right,
                    var(--accent-color-brighter) 0%,
                    var(--accent-color-brighter) var(--progress),
                    var(--background-mid) var(--progress),
                    var(--background-mid) 100%
                );
            }
        }
    }

    input[type="range"]::-webkit-slider-thumb {
        appearance: none;
    }

    input[type="range"]::-webkit-slider-runnable-track {
        appearance: none;
        box-shadow: none;
        border: none;
    }

    .can-seek {
        & {
            cursor: pointer;
        }

        &::-webkit-slider-thumb {
            cursor: ew-resize;
            height: 10px;
            width: 4px;
            border-radius: 2px;
            background: var(--root-color);
            transition: background .3s ease-in-out;
        }

        &::-webkit-slider-runnable-track {
            cursor: pointer;
        }

        @media (prefers-color-scheme: light) {
            &::-webkit-slider-thumb {
                background: var(--accent-color-dim);
            }
        }
    }
</style>