import { get, derived, type Readable } from "svelte/store";

import {
    amplifierMuteToggle,
    amplifierVolumeDown,
    amplifierVolumeSet,
    amplifierVolumeUp,
    muteOff,
    muteOn,
    volumeStepChange,
    volumeStepSet,
} from "./commands.ts";
import {
    amplifierState,
    isAmplifierPowerOn,
    isCbusAmpModeEnabled,
    isHandlingAmplifier,
    isPowerOn,
    isPreAmpModeEnabled,
    systemInfo,
    zoneState,
} from "./state.ts";

export enum PreAmpHardware {
    Amplifier,
    StreamerPreAmp, // StreamMagic streamer with Pre-Amp mode enabled
    StreamerControlBus, // StreamMagic control bus only appears to support volume up/down
}

type PreAmpState = {
    hardware: PreAmpHardware | undefined;
    volume: number | undefined;
    volumeStep: number | undefined;
    isMuted: boolean | undefined;
    isPoweredOn: boolean | undefined;
};

export enum VolumeChangeDegree {
    Big,
    Small,
}

const STREAMER_MODELS_SUPPORTING_100_STEPS = ["MXN10"];

/**
 * Manages pre-amp related state and actions.
 *
 * Abstracts away the distinction between the Streamer and Amplifier Pre-Amp interfaces. The rest
 * of the app should be able to invoke these actions and not be concerned with whether the Pre-Amp
 * hardware is the StreamMagic Streamer or an Amplifier.
 */
const preAmpManager = () => {
    // Derived state to track the current pre-amp-related properties. Priority is given to
    // Amplifier state, with a fallback to the Streamer state if available.
    const preAmpState: Readable<PreAmpState | undefined> = derived(
        [
            amplifierState,
            isCbusAmpModeEnabled,
            isHandlingAmplifier,
            isPowerOn,
            isPreAmpModeEnabled,
            zoneState,
        ],
        ([
            amplifierState,
            isCbusAmpModeEnabled,
            isHandlingAmplifier,
            isPowerOn,
            isPreAmpModeEnabled,
            zoneState,
        ]) => {
            // Determine PreAmp hardware in specific priority order. Amplifier first, then
            // StreamerPreAmp, then StreamerControlBus.

            const hardware = isHandlingAmplifier
                ? PreAmpHardware.Amplifier
                : isPreAmpModeEnabled
                  ? PreAmpHardware.StreamerPreAmp
                  : isCbusAmpModeEnabled
                    ? PreAmpHardware.StreamerControlBus
                    : undefined;

            if (hardware === PreAmpHardware.Amplifier) {
                return {
                    hardware,
                    volume: amplifierState?.volume ?? undefined,
                    volumeStep: undefined,
                    isMuted: amplifierState?.is_muted ?? undefined,
                    isPoweredOn: amplifierState?.is_powered_on ?? undefined,
                };
            } else if (hardware === PreAmpHardware.StreamerPreAmp) {
                return {
                    hardware,
                    volume: zoneState?.volume_percent ?? undefined,
                    volumeStep: zoneState?.volume_step ?? undefined,
                    isMuted: zoneState?.mute ?? undefined,
                    isPoweredOn: isPowerOn ?? undefined,
                };
            } else if (hardware === PreAmpHardware.StreamerControlBus) {
                return {
                    hardware,
                    volume: undefined,
                    volumeStep: undefined,
                    isMuted: undefined,
                    isPoweredOn: isPowerOn ?? undefined,
                };
            } else {
                return {
                    hardware: undefined,
                    volume: undefined,
                    volumeStep: undefined,
                    isMuted: undefined,
                    isPoweredOn: undefined,
                };
            }
        },
    );

    // --------------------------------------------------------------------------------------------
    // Internal helpers
    // --------------------------------------------------------------------------------------------

    // StreamMagic appears to support 30 volume steps (each being ~3%) for some models, and 100
    // steps (i.e. percentage) for others. Amplifiers are assumed to support 0-100 percentages.
    // Streamers which support 100 steps will actually use the percentage-based change amounts.

    const smallVolumeStepChangeAmount = 1;
    const bigVolumeStepChangeAmount = 2;
    const minVolumeSteps = 0;
    const maxVolumeSteps = 30;

    const smallVolumePercentChangeAmount = 1;
    const bigVolumePercentChangeAmount = 5;
    const minVolumePercent = 0;
    const maxVolumePercent = 100;

    /**
     * Determine whether the current streamer device supports 100 steps (i.e. percentages). Defaults
     * to false, unless the device is explicitly included in STREAMER_MODELS_SUPPORTING_100_STEPS
     * *or* the device is returning a volume_step which matches the volume_percent (which can only
     * be true if steps == percent).
     */
    const streamerSupports100Steps = derived(
        [zoneState, systemInfo],
        ([$zoneState, $systemInfo]) => {
            const haveVolumeStep = typeof $zoneState?.volume_step === "number";
            const haveVolumePercent = typeof $zoneState?.volume_percent === "number";
            const model = $systemInfo?.model;

            if (model && STREAMER_MODELS_SUPPORTING_100_STEPS.includes(model)) {
                return true;
            }

            // Apply our heuristics to determine whether the streamer supports 100 steps.
            if (!isPreAmpModeEnabled || !$zoneState) {
                return false;
            } else if (haveVolumeStep && haveVolumePercent) {
                if ($zoneState.volume_step === 0 && $zoneState.volume_percent === 0) {
                    // Need > 0 values for the steps == percent check to be valid.
                    return false;
                } else {
                    // Streamers which return the same value for step and percent are assumed to support
                    // the full 100 steps (i.e. percentages).
                    return $zoneState.volume_step === $zoneState.volume_percent;
                }
            } else if (haveVolumeStep) {
                return false;
            } else if (haveVolumePercent) {
                return true;
            }

            return false;
        },
    );

    /**
     * Calculate the small and big volume change amounts based on the current volume level.
     * The Streamer uses "steps" to control volume (of which there are 0-30 steps); and the
     * Amplifier uses a percentage (0-100). Step changes are either 1 or 2 (small/big), and
     * percentage changes are either 1% or 5% (small/big).
     */
    const calculateVolumeChanges = () => {
        const state = get(preAmpState);
        const hardware = state?.hardware;
        const volume = state?.volume;
        const volumeStep = state?.volumeStep;

        if (typeof hardware === "undefined") {
            return;
        }

        const streamerSupportsPercentages = get(streamerSupports100Steps);

        if (
            streamerSupportsPercentages ||
            get(preAmpState)?.hardware === PreAmpHardware.Amplifier
        ) {
            // We have a PreAmp device (streamer or amplifier) which supports 0-100 percentage-based
            // volume controls.

            const currentVolume = volume;

            return {
                [VolumeChangeDegree.Small]: {
                    up:
                        typeof currentVolume !== "undefined"
                            ? Math.min(
                                  currentVolume + smallVolumePercentChangeAmount,
                                  maxVolumePercent,
                              )
                            : 0,
                    down:
                        typeof currentVolume !== "undefined"
                            ? Math.max(
                                  currentVolume - smallVolumePercentChangeAmount,
                                  minVolumePercent,
                              )
                            : 0,
                },
                [VolumeChangeDegree.Big]: {
                    up:
                        typeof currentVolume !== "undefined"
                            ? Math.min(
                                  currentVolume + bigVolumePercentChangeAmount,
                                  maxVolumePercent,
                              )
                            : 0,
                    down:
                        typeof currentVolume !== "undefined"
                            ? Math.max(
                                  currentVolume - bigVolumePercentChangeAmount,
                                  minVolumePercent,
                              )
                            : 0,
                },
            };
        } else if (hardware === PreAmpHardware.StreamerPreAmp) {
            // We have a streamer which support expects 0-30 step-based volume controls.

            const currentStep = volumeStep;

            return {
                [VolumeChangeDegree.Small]: {
                    up:
                        typeof currentStep !== "undefined"
                            ? Math.min(currentStep + smallVolumeStepChangeAmount, maxVolumeSteps)
                            : 0,
                    down:
                        typeof currentStep !== "undefined"
                            ? Math.max(currentStep - smallVolumeStepChangeAmount, minVolumeSteps)
                            : 0,
                },
                [VolumeChangeDegree.Big]: {
                    up:
                        typeof currentStep !== "undefined"
                            ? Math.min(currentStep + bigVolumeStepChangeAmount, maxVolumeSteps)
                            : 0,
                    down:
                        typeof currentStep !== "undefined"
                            ? Math.max(currentStep - bigVolumeStepChangeAmount, minVolumeSteps)
                            : 0,
                },
            };
        } else if (hardware === PreAmpHardware.StreamerControlBus) {
            // Control Bus has no awareness of current volume, and only supports increasing and
            // decreasing the volume by one unit.

            return {
                [VolumeChangeDegree.Small]: {
                    up: 1,
                    down: -1,
                },
                [VolumeChangeDegree.Big]: {
                    up: 1,
                    down: -1,
                },
            };
        }
    };

    /**
     * Sets a new volume level by the given degree, for the given direction.
     *
     * @param degree Degree of the volume change to make.
     * @param direction Whether the volume change is up or down.
     */
    const setVolume = async (degree: VolumeChangeDegree, direction: "down" | "up") => {
        const state = get(preAmpState);
        const hardware = state?.hardware;
        const volumeChanges = calculateVolumeChanges();

        if (typeof hardware === "undefined" || typeof volumeChanges === undefined || !state?.isPoweredOn) {
            return;
        }

        if (hardware === PreAmpHardware.Amplifier) {
            if (degree === VolumeChangeDegree.Small) {
                // Small amplifier changes skip the volumeChanges approach and instead fall back on
                // the "VolumeUp" and "VolumeDown" amplifier functions.
                if (direction === "down") {
                    await amplifierVolumeDown();
                } else {
                    await amplifierVolumeUp();
                }
            } else {
                await amplifierVolumeSet(volumeChanges[degree][direction]);
            }
        } else if (hardware === PreAmpHardware.StreamerPreAmp) {
            await volumeStepSet(volumeChanges[degree][direction]);
        } else if (hardware === PreAmpHardware.StreamerControlBus) {
            await volumeStepChange(volumeChanges[degree][direction]);
        }
    };

    // --------------------------------------------------------------------------------------------
    // Exposed Pre-Amp control functions
    // --------------------------------------------------------------------------------------------

    /**
     * Toggle mute on/off.
     */
    const toggleMute = async () => {
        const state = get(preAmpState);
        const hardware = state?.hardware;
        const isMuted = state?.isMuted;

        if (typeof hardware === "undefined" || typeof isMuted === "undefined" || !state?.isPoweredOn) {
            return;
        }

        if (hardware === PreAmpHardware.Amplifier) {
            await amplifierMuteToggle();
        } else if (hardware === PreAmpHardware.StreamerPreAmp) {
            if (isMuted) {
                await muteOff();
            } else if (!isMuted) {
                await muteOn();
            }
        }

        // Note: This is a no-op for PreAmpHardware.StreamerControlBus.
    };

    /**
     * Decrease the volume.
     *
     * @param degree Degree of the volume decrease.
     */
    const volumeDown = async (degree: VolumeChangeDegree) => {
        await setVolume(degree, "down");
    };

    /**
     * Increase the volume.
     *
     * @param degree Degree of the volume increase.
     */
    const volumeUp = async (degree: VolumeChangeDegree) => {
        await setVolume(degree, "up");
    };

    // --------------------------------------------------------------------------------------------

    return {
        subscribe: preAmpState.subscribe,
        toggleMute,
        volumeDown,
        volumeUp,
    };
};

export default preAmpManager;
