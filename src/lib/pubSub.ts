import { listen } from "@tauri-apps/api/event";

import { appLogs, DEV_MODE, isAppOpen, isRepeatEnabled, isShuffleEnabled } from "./state.ts";
import type { AppLog } from "../types/generated/AppLog.ts";

/**
 * PubSub manager.
 *
 * Source: https://gist.github.com/sidola/eaf987d8c4c7e8445b61dc07c33a842f
 */
class PubSub<T> {
    private eventHandlers: { [key: string]: any[] } = {};

    public subscribe<E extends keyof T & string>(event: E, callback: (message: T[E]) => void) {
        const list = this.eventHandlers[event] ?? [];
        list.push(callback);
        this.eventHandlers[event] = list;

        return () => this.unsubscribe(event, callback);
    }

    public unsubscribe<E extends keyof T & string>(event: E, callback: (message: T[E]) => void) {
        let list = this.eventHandlers[event] ?? [];
        list = list.filter((h) => h !== callback);
        this.eventHandlers[event] = list;
    }

    public publish<E extends keyof T & string>(event: E, message?: T[E]) {
        // TODO: `message` is optional to allow for events which have no payloads. This removes
        //  TypeScript's ability to inform the caller of `publish()` if they're not passing a
        //  required non-undefined payload. See if this can be improved.
        DEV_MODE && console.log(`publishing: ${event}`, message);
        this.eventHandlers[event]?.forEach((h) => h(message));
    }
}

// ================================================================================================

// ------------------------------------------------------------------------------------------------
// PunyTunes PubSub Event definitions

export type Events = {
    AppErrorLog: AppLog;
    AppLostFocus: undefined;
    AppResizeComplete: undefined;
    DetailViewUpdated: undefined;
    RepeatModeSet: boolean;
    ScrollToCurrentPresetItem: undefined;
    ScrollToCurrentQueueItem: undefined;
    ShuffleModeSet: boolean;
    TrayAppClosed: undefined;
    TrayAppOpened: undefined;
    TrayLeftClick: undefined;
};

const pubSub = new PubSub<Events>();

// ------------------------------------------------------------------------------------------------
// Publish PubSub events based on state changes.
//
// These are intended for use when Svelte's subscribe() is problematic; such as when a subscriber
// does not wish to receive the initial state and only wishes to receive subsequent updates.

let latestAppErrorTime: bigint = BigInt(0);

appLogs.subscribe(($appLogs) => {
    const latestError = $appLogs.find((log) => log.level === "error");

    if (latestError && latestError.when > latestAppErrorTime) {
        latestAppErrorTime = latestError.when;
        pubSub.publish("AppErrorLog", latestError);
    }
});

isRepeatEnabled.subscribe((isEnabled) => pubSub.publish("RepeatModeSet", isEnabled));

isShuffleEnabled.subscribe((isEnabled) => pubSub.publish("ShuffleModeSet", isEnabled));

await listen("tray-left-click", () => pubSub.publish("TrayLeftClick"));

await listen("app-lost-focus", () => pubSub.publish("AppLostFocus"));

pubSub.subscribe("TrayAppOpened", () => isAppOpen.set(true));
pubSub.subscribe("TrayAppClosed", () => isAppOpen.set(false));


// ================================================================================================

export default pubSub;
