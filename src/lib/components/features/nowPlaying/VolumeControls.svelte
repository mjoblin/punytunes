<script lang="ts">
    import {
        IconArrowNarrowDown,
        IconArrowNarrowUp,
        IconArrowsDown,
        IconArrowsUp,
        IconSquareRoundedMinusFilled,
        IconSquareRoundedPlusFilled,
        IconVolume,
        IconVolumeOff,
    } from "@tabler/icons-svelte";

    import preAmpManager, { PreAmpHardware, VolumeChangeDegree } from "../../../preAmpManager.ts";
    import Arc from "../../dataDisplay/Arc.svelte";
    import IconButton from "../../buttons/IconButton.svelte";

    const preAmp = preAmpManager();

    $: isUsingControlBus = $preAmp?.hardware === PreAmpHardware.StreamerControlBus;
    $: volume = $preAmp?.volume;
    $: isMuted = $preAmp?.isMuted;
    $: isPoweredOn = $preAmp?.isPoweredOn;
</script>

<div class="VolumeControls" class:disabled={!isPoweredOn}>
    {#if isUsingControlBus}
        <!-- Volume control -->
        <div class="up-down-buttons">

            <!-- Change volume (small) -->
            <div class="button-pair-control-bus">
                <IconButton
                    disabled={isMuted || !isPoweredOn}
                    size={18}
                    padding="0"
                    icon={IconSquareRoundedPlusFilled}
                    on:click={() => preAmp.volumeUp(VolumeChangeDegree.Small)}
                />
                <IconButton
                    disabled={isMuted || !isPoweredOn}
                    size={18}
                    padding="0"
                    icon={IconSquareRoundedMinusFilled}
                    on:click={() => preAmp.volumeDown(VolumeChangeDegree.Small)}
                />
            </div>
        </div>
    {:else}
        <!-- Mute button -->
        <IconButton
            disabled={!isPoweredOn}
            icon={isMuted ? IconVolumeOff : IconVolume}
            size={14}
            on:click={preAmp.toggleMute}
        />

        <!-- Current volume arc -->
        <div class="current-volume-arc" class:disabled={!isPoweredOn}>
            <Arc
                radius={20}
                rotation={180}
                startEndGap={70}
                thickness={4}
                progress={typeof volume === "undefined" ? 0 : 360 * (volume / 100)}
                color="--goldenrod"
                trackColor="--background-mid"
            >
                <span class="current-volume-level">{volume ?? ""}</span>
            </Arc>
        </div>

        <!-- Volume control -->
        <div class="up-down-buttons">

            <!-- Change volume (small) -->
            <div class="button-pair">
                <IconButton
                    disabled={isMuted || !isPoweredOn}
                    size={10}
                    icon={IconArrowNarrowUp}
                    on:click={() => preAmp.volumeUp(VolumeChangeDegree.Small)}
                />
                <IconButton
                    disabled={isMuted || !isPoweredOn}
                    size={10}
                    icon={IconArrowNarrowDown}
                    on:click={() => preAmp.volumeDown(VolumeChangeDegree.Small)}
                />
            </div>

            <!-- Change volume (larger) -->
            <div class="button-pair">
                <IconButton
                    disabled={isMuted || !isPoweredOn}
                    size={10}
                    icon={IconArrowsUp}
                    on:click={() => preAmp.volumeUp(VolumeChangeDegree.Big)}
                />
                <IconButton
                    disabled={isMuted || !isPoweredOn}
                    size={10}
                    icon={IconArrowsDown}
                    on:click={() => preAmp.volumeDown(VolumeChangeDegree.Big)}
                />
            </div>
        </div>
    {/if}
</div>

<style>
    .VolumeControls {
        display: flex;
        gap: 0.25em;
        align-items: center;
        margin-top: 10px;

        &.disabled {
            opacity: 0.60;

            @media (prefers-color-scheme: light) {
                opacity: 1;
            }
        }
    }

    .current-volume-arc {
        &.disabled {
            @media (prefers-color-scheme: light) {
                opacity: 0.45;
            }
        }
    }

    .current-volume-level {
        font-size: 0.85em;
        font-weight: 600;
        opacity: 0.8;
        color: var(--text-normal);
    }

    .up-down-buttons {
        display: flex;
        gap: 0;
    }

    .button-pair {
        display: flex;
        flex-direction: column;
        gap: 0;
    }

    .button-pair-control-bus {
        display: flex;
        flex-direction: column;
        gap: 0.15em;
    }
</style>