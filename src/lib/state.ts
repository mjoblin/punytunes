import type { Updater } from "svelte/store";
import { derived, get, writable } from "svelte/store";
import type { UpdateManifest, UpdateStatus } from "@tauri-apps/api/updater";
import type { OsType } from "@tauri-apps/api/os";
import { listen } from "@tauri-apps/api/event";
import { type as getOsType } from "@tauri-apps/api/os";

import { getUserSetting, persistedUserSettingFactory, setUserSetting } from "./userSettings.ts";
import { uiReady } from "./commands";
import createThemeManager from "./themeManager.ts";

// NOTE: The types with the "Streamer" prefix are the top-level types. They wrap the sub-type
//  inside a "data" key. The wrapper exists to allow for the "zone" key to also exist when the
//  payload data is a zone-related payload. For now, the zone is ignored and just the sub-type
//  is exposed to the rest of the app.
import type { AppLog } from "../types/generated/AppLog.ts";
import type { Level } from "../types/generated/Level.ts";
import type { StreamMagicManagerStateMsg } from "../types/generated/StreamMagicManagerStateMsg.ts";
import type { StreamMagicDevice } from "../types/generated/StreamMagicDevice.ts";
import type { WebSocketClientStatus } from "../types/generated/WebSocketClientStatus.ts";

import type { StreamerPresets } from "../types/generated/streammagic_payloads/StreamerPresets.ts";
import type { StreamerQueueList } from "../types/generated/streammagic_payloads/StreamerQueueList.ts";
import type { StreamerSystemPower } from "../types/generated/streammagic_payloads/StreamerSystemPower.ts";
import type { StreamerZoneNowPlaying } from "../types/generated/streammagic_payloads/StreamerZoneNowPlaying.ts";
import type { StreamerZonePlayState } from "../types/generated/streammagic_payloads/StreamerZonePlayState.ts";
import type { StreamerZonePosition } from "../types/generated/streammagic_payloads/StreamerZonePosition.ts";

import type { Presets } from "../types/generated/streammagic_payloads/Presets.ts";
import type { QueueList } from "../types/generated/streammagic_payloads/QueueList.ts";
import type { SystemPower } from "../types/generated/streammagic_payloads/SystemPower.ts";
import type { SystemSources } from "../types/generated/streammagic_payloads/SystemSources.ts";
import type { ZoneNowPlaying } from "../types/generated/streammagic_payloads/ZoneNowPlaying.ts";
import type { ZonePlayState } from "../types/generated/streammagic_payloads/ZonePlayState.ts";
import type { ZonePosition } from "../types/generated/streammagic_payloads/ZonePosition.ts";
import type { ZoneState } from "../types/generated/streammagic_payloads/ZoneState.ts";

import type { StreamerSystemSources } from "../types/generated/streammagic_payloads/StreamerSystemSources.ts";
import type { StreamerZoneState } from "../types/generated/streammagic_payloads/StreamerZoneState.ts";
import type { SystemInfo } from "../types/generated/streammagic_payloads/SystemInfo.ts";
import type { StreamerSystemInfo } from "../types/generated/streammagic_payloads/StreamerSystemInfo.ts";

import type { AmplifierManagerStateMsg } from "../types/generated/AmplifierManagerStateMsg.ts";
import type { AmplifierState } from "../types/generated/AmplifierState.ts";

// ------------------------------------------------------------------------------------------------
// Exported Svelte state
// ------------------------------------------------------------------------------------------------

export const DEV_MODE = import.meta.env.DEV;

const IS_BUFFERING_DELAY = 2000;
const MAX_APP_LOGS = 1000;

export let osType = writable<OsType>(await getOsType());

export let alwaysUseArtRustClient = writable<boolean>(false);

// ------------------------------------------------------------------------------------------------
// App-level state (not specific to the streamer)

// Theme

export const themeManager = createThemeManager();

export type DetailsPosition = "bottom" | "top" | undefined;

// UI application detailViews
export type DetailsView =
    | "dev"
    | "info"
    | "logs"
    | "payloads"
    | "presets"
    | "queue"
    | "sources"
    | "streamer"
    | undefined;

export type AppUpdateState = {
    checkForUpdateError: string | undefined;
    isCheckingForUpdate: boolean;
    isUpdateAvailable: boolean;
    manifest: UpdateManifest | undefined;
    updateError: string | undefined;
    updateStatus: UpdateStatus | undefined;
};

export let appUpdateState = writable<AppUpdateState>({
    checkForUpdateError: undefined,
    isCheckingForUpdate: false,
    isUpdateAvailable: false,
    manifest: undefined,
    updateError: undefined,
    updateStatus: undefined,
});

// The "info" details view serves two needs: information, and user settings. This feels a bit
// overloaded, and could be changed in the future.
export let currentInfoView = writable<"info" | "userSettings">("info");

export let uiInitialized = writable<boolean>(false);

export let isAppOpen = writable<boolean>(false);

export let appLogs = writable<AppLog[]>([]);

export let selectedPayload = writable<string>("nowPlaying");

export let detailsScrollPositions = writable<Record<Exclude<DetailsView, undefined>, number>>({
    dev: 0,
    info: 0,
    logs: 0,
    payloads: 0,
    presets: 0,
    queue: 0,
    sources: 0,
    streamer: 0,
});

export let logLevelDisplay = writable<Record<Level, boolean>>({
    error: true,
    warn: true,
    info: true,
    debug: false,
    trace: false,
});

// ------------------------------------------------------------------------------------------------
// App-level state which is also persisted between sessions as user settings

const activeDetailsViewImpl = persistedUserSettingFactory<DetailsView>(
    await getUserSetting("activeDetailsView"),
    async (value: DetailsView) => {
        await setUserSetting("activeDetailsView", value);
    },
);

export let activeDetailsView = await activeDetailsViewImpl();

const detailsPositionImpl = persistedUserSettingFactory<DetailsPosition>(
    await getUserSetting("detailsPosition"),
    async (value: DetailsPosition) => {
        await setUserSetting("detailsPosition", value);
    },
);

export let detailsPosition = await detailsPositionImpl();

const showDetailedQueueImpl = persistedUserSettingFactory<boolean>(
    (await getUserSetting("queueDisplay")) === "detailed",
    async (value: boolean) => {
        await setUserSetting("queueDisplay", value ? "detailed" : "simple");
    },
);

export let showDetailedQueue = await showDetailedQueueImpl();

const showDetailedPresetsImpl = persistedUserSettingFactory<boolean>(
    (await getUserSetting("presetsDisplay")) === "detailed",
    async (value: boolean) => {
        await setUserSetting("presetsDisplay", value ? "detailed" : "simple");
    },
);

export let showDetailedPresets = await showDetailedPresetsImpl();

const showVolumeControlsImpl = persistedUserSettingFactory<boolean>(
    await getUserSetting("showVolumeControls"),
    async (value: boolean) => {
        await setUserSetting("showVolumeControls", value);
    },
);

export let showVolumeControls = await showVolumeControlsImpl();

const followCurrentQueueItemImpl = persistedUserSettingFactory<boolean>(
    await getUserSetting("followCurrentQueueItem"),
    async (value: boolean) => {
        await setUserSetting("followCurrentQueueItem", value);
    },
);

export let followCurrentQueueItem = await followCurrentQueueItemImpl();

// ------------------------------------------------------------------------------------------------
// Store the StreamMagicManager state from Rust. This gets broken up into different derived state
// chunks for the UI to consume.

let streamMagicManagerState = writable<StreamMagicManagerStateMsg | undefined>();

type Devices = {
    discovered: StreamMagicDevice[];
    is_activating: boolean;
    is_discovering: boolean;
    is_testing_connection: boolean;
};

export const devices = derived(streamMagicManagerState, ($streamMagicManagerState) => {
    return {
        discovered: $streamMagicManagerState?.devices || [],
        is_activating: $streamMagicManagerState?.is_activating ?? false,
        is_discovering: $streamMagicManagerState?.is_discovering ?? false,
        is_testing_connection: $streamMagicManagerState?.is_testing_connection ?? false,
    } as Devices;
});

export const webSocketClientStatus = derived(
    streamMagicManagerState,
    ($streamMagicManagerState) =>
        ($streamMagicManagerState?.websocket_client_status || {
            state: "Disconnected",
            metadata: null,
        }) as WebSocketClientStatus,
);

// ------------------------------------------------------------------------------------------------
// Store the various StreamMagic-specific payloads. These have been stripped of their
// WithZone/WithoutZone details as the UI components currently don't care about zones.

export const resetAllStreamerState = () => {
    nowPlaying.set(undefined);
    playState.set(undefined);
    positionInternal.set(undefined);
    presets.set(undefined);
    queueList.set(undefined);
    systemPowerInternal.set(undefined);
    systemSources.set(undefined);
    zoneState.set(undefined);
};

export let nowPlaying = writable<ZoneNowPlaying | undefined>();

export let playState = writable<ZonePlayState | undefined>();

export let positionInternal = writable<ZonePosition | undefined>();
export const position = derived(
    positionInternal,
    ($positionInternal) => $positionInternal?.position,
);

export let presets = writable<Presets | undefined>();

export let queueList = writable<QueueList | undefined>();

export let systemInfo = writable<SystemInfo | undefined>();

// SystemPower and ZonePosition are exposed to UI components in a simplified derived form.
export let systemPowerInternal = writable<SystemPower | undefined>();
export const power = derived(
    systemPowerInternal,
    ($systemPowerInternal) => $systemPowerInternal?.power,
);

export const systemSources = writable<SystemSources | undefined>();

export let zoneState = writable<ZoneState | undefined>();

/**
 * Compute the current position as a normalized value between 0-1.
 */
export const positionNormalized = derived([position, playState], ([$position, $playState]) => {
    const duration = $playState?.metadata?.duration;

    if (typeof $position === "number" && typeof duration === "number") {
        if (duration === 0) {
            return undefined;
        } else {
            return $position / duration;
        }
    } else {
        return undefined;
    }
});

export let wasConnectedAtAppHide = writable<boolean>(false);

// ------------------------------------------------------------------------------------------------
// Store the AmplifierManager state from Rust.

export let amplifierManagerState = writable<AmplifierManagerStateMsg | undefined>();

export let amplifierState = writable<AmplifierState | undefined>();

// ------------------------------------------------------------------------------------------------
// Derived state helpers

export const isActivating = derived(streamMagicManagerState, ($streamMagicManagerState) => !!$streamMagicManagerState?.is_activating);

export const isCbusAmpModeEnabled = derived(zoneState, ($zoneState) => !!($zoneState?.cbus && ["amplifier", "receiver"].includes($zoneState.cbus)));

export const isPreAmpModeEnabled = derived(zoneState, ($zoneState) => !!($zoneState?.pre_amp_mode && $zoneState.pre_amp_state === "on"));

export const isPowerOn = derived(power, ($powerState) => $powerState === "ON");

export const isInStandby = derived(nowPlaying, ($nowPlaying) => $nowPlaying?.source?.id === "IDLE");

export const isHandlingAmplifier = derived(amplifierManagerState, ($amplifierManagerState) => !!$amplifierManagerState?.is_handling_amplifier);

export const isAmplifierPowerOn = derived(amplifierState, ($amplifierState) => !!$amplifierState?.is_powered_on);

// ------------------------------------------------------------------------------------------------
// Handle setting isConnected. This is not a naive equivalent of $webSocketClientStatus.state as
// it needs to remain unchanged while a connection test is pending. This is because the connection
// test can result in the state becoming Disconnected and then Connected in rapid succession,
// which causes an app flash. So the goal here is to only set isConnected once any pending
// connection test has completed.
//
// Approach: Track test pending state in isConnectionTestPending. When the app is opened,
// immediately enter test-pending state. Remain in test-pending state until the WebSocketClient's
// "is_testing_connection" changes back to false. Do not update isConnected while test is pending.

export const isConnectionTestPending = writable<boolean>(false);

const isTestingConnection = derived(
    streamMagicManagerState,
    ($streamMagicManagerState) => !!$streamMagicManagerState?.is_testing_connection,
);

// NOTE: The testStreamerConnection() command (commands.ts) will set isConnectionTestPending to true
isAppOpen.subscribe((isOpen) => !isOpen && isConnectionTestPending.set(false));

isTestingConnection.subscribe((isTesting) => {
    if (get(isConnectionTestPending) && !isTesting) {
        isConnectionTestPending.set(false);
    }
});

export const isConnected = derived(
    [webSocketClientStatus, isConnectionTestPending],
    ([$webSocketClientStatus, $isConnectionTestPending], set) => {
        if (!$isConnectionTestPending) {
            set(["Connected", "TestingConnection"].includes($webSocketClientStatus.state));
        }
    },
);

// When true, the app is actively attempting to figure out if the connection is established.
// When true, consumers should consider the app to be in a "might be, might not be" connected
// state.
export const isEstablishingConnectionState = derived(
    [isTestingConnection, isActivating], ([$isTestingConnection, $isActivating]) => $isTestingConnection || $isActivating
);

// isConnected handling complete -----------------------------------------------------------------------------

export const isPlaying = derived(playState, ($playState) => $playState?.state === "play");

export const isBufferingAudio = derived(
    playState,
    ($playState, setIsBuffering) => {
        let isCurrentlyBuffering = $playState?.state === "buffering";

        let timeoutId = setTimeout(
            () => {
                setIsBuffering(isCurrentlyBuffering);
            },
            isCurrentlyBuffering ? IS_BUFFERING_DELAY : 0,
        );

        return () => {
            clearTimeout(timeoutId);
        };
    },
    false,
);

export const isConnecting = derived(playState, ($playState) => $playState?.state === "connecting");

export const isRepeatEnabled = derived(
    playState,
    ($playState) => $playState?.mode_repeat === "all",
);

export const isShuffleEnabled = derived(
    playState,
    ($playState) => $playState?.mode_shuffle === "all",
);

export const streamerDisplay = derived(nowPlaying, ($nowPlaying) => $nowPlaying?.display);

export const activeControls = derived(nowPlaying, ($nowPlaying) => $nowPlaying?.controls || []);

// The active source is found in a few places; arbitrarily choosing nowPlaying as the canonical one
export const activeSourceId = derived(nowPlaying, ($nowPlaying) => $nowPlaying?.source?.id);

// ------------------------------------------------------------------------------------------------

/**
 * Initialize the state-related components of the application.
 *
 *   - Set up listeners to receive the various message types emitted from Rust. These message
 *     payloads are used to populate Svelte state.
 *   - Invoke Rust's uiReady command when the UI is ready to receive messages.
 */
const initialize = async () => {
    // Listen for general application messages

    await listen<AppLog>("AppLog", (message) => {
        appLogs.update((priorState: AppLog[]) =>
            [message.payload, ...priorState].toSpliced(MAX_APP_LOGS, 1),
        );
    });

    await listen<StreamMagicManagerStateMsg>("StreamMagicManagerState", (message) => {
        const managerState = message.payload;

        streamMagicManagerState.set(managerState);

        // If we get a Disconnected state while not in activation/test mode, then we want
        // to clear any prior streamer state so as to leave the UI in a default
        // nothing-to-display state.
        !managerState.is_activating &&
            !managerState.is_testing_connection &&
            managerState.websocket_client_status.state === "Disconnected" &&
            resetAllStreamerState();
    });

    // Listen for StreamMagic payload messages. When the payloads are used to set Svelte state,
    // the WithZone/WithoutZone details are ignored and the nested payload (stored in the
    // "data" key) is used to set the Svelte state.

    await listen<StreamerQueueList>("StreamerQueueList", (message) => {
        queueList.set(message.payload.data);
    });

    await listen<StreamerPresets>("StreamerPresets", (message) => {
        presets.set(message.payload.data);
    });

    await listen<StreamerSystemInfo>("StreamerSystemInfo", (message) => {
        systemInfo.set(message.payload.data);
    });

    await listen<StreamerSystemPower>("StreamerSystemPower", (message) => {
        systemPowerInternal.set(message.payload.data);
    });

    await listen<StreamerSystemSources>("StreamerSystemSources", (message) => {
        systemSources.set(message.payload.data);
    });

    await listen<StreamerZoneNowPlaying>("StreamerZoneNowPlaying", (message) => {
        nowPlaying.set(message.payload.data);
    });

    await listen<StreamerZonePlayState>("StreamerZonePlayState", (message) => {
        playState.set(message.payload.data);
    });

    await listen<StreamerZonePosition>("StreamerZonePosition", (message) => {
        // StreamerZonePosition is unique in that we allow for a missing payload (which is not
        // considered an error state).
        positionInternal.set(message.payload?.data);
    });

    await listen<StreamerZoneState>("StreamerZoneState", (message) => {
        // zoneState.set(message.payload.data);

        // If the power state or active input has changed, then set the playhead position to
        // undefined. This ensures the UI will show a more complete "reset" state when the streamer
        // has undergone a significant state change.

        const zoneStateUpdater: Updater<ZoneState | undefined> = (
            priorZoneState: ZoneState | undefined,
        ) => {
            const newZoneState = message.payload.data;

            // Reset the playhead position to null whenever the streamer enters an "I'm starting
            // playback from scratch in some way" mode. Currently this means when the streamer has
            // just been powered on, or the audio source has changed.
            if (
                priorZoneState?.power !== newZoneState?.power ||
                priorZoneState?.source !== newZoneState?.source
            ) {
                positionInternal.set(undefined);
            }

            return newZoneState;
        };

        zoneState.update(zoneStateUpdater);
    });

    // Listen for Amplifier payload messages. When the payloads are used to set Svelte state,

    await listen<AmplifierManagerStateMsg>("AmplifierManagerState", (message) => {
        amplifierManagerState.set(message.payload);
        console.log("AmplifierManagerState", message.payload);
    });

    await listen<AmplifierState>("AmplifierState", (message) => {
        amplifierState.set(message.payload);
        console.log("AmplifierState", message.payload);
    });

    // Inform the Rust side that the UI is ready to proceed. This will trigger a full emission of
    // various state messages, allowing the Svelte state to populate.
    await uiReady();

    // Inform the rest of the UI that the UI is initialized.
    uiInitialized.set(true);
};

await initialize();
