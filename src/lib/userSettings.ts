import { writable } from "svelte/store";
import { Store } from "tauri-plugin-store-api";
import { type as getOsType } from "@tauri-apps/api/os";

import type { DetailsPosition, DetailsView } from "./state.ts";

type DetailDisplay = "detailed" | "simple";

// Supported user settings
type UserSettings = {
    activeDetailsView: DetailsView;
    detailsPosition: DetailsPosition;
    followCurrentQueueItem: boolean;
    presetsDisplay: DetailDisplay;
    queueDisplay: DetailDisplay;
    showVolumeControls: boolean;
};

// User setting defaults
const defaults: UserSettings = {
    activeDetailsView: undefined,
    detailsPosition: "bottom",
    followCurrentQueueItem: true,
    presetsDisplay: "simple",
    queueDisplay: "simple",
    showVolumeControls: true,
};

const store = new Store("user-settings.json");

/**
 * Set the given UserSettings key to the given value, and persist it to storage.
 */
export const setUserSetting = async <K extends keyof UserSettings>(
    key: K,
    value: UserSettings[K],
) => {
    try {
        await store.set(key, typeof value === "undefined" ? null : value);
        await store.save();
    } catch (e) {
        console.error("userSettings get error", e);
    }
};

/**
 * Get the value of the given UserSettings key, falling back on the default if there's no known value.
 */
export const getUserSetting = async <K extends keyof UserSettings>(
    key: K,
): Promise<UserSettings[K]> => {
    try {
        // @ts-ignore
        let result: UserSettings[K] = await store.get<UserSettings[K]>(key);

        if (result === null) {
            if (key === "detailsPosition") {
                // This special case is a little unfortunate, but it's here to ensure that we
                // absolutely default the detailsPosition value to a reasonable OS-specific default,
                // which we can't do until we know what the OS is -- and due to the timing of
                // when startup things happen, it might be risky to rely on osType from the Svelte
                // state.
                const osType = await getOsType();
                // @ts-ignore
                result = osType === "Windows_NT" ? "top" : "bottom";
                await setUserSetting("detailsPosition", result);
            } else {
                result = defaults[key];
                await setUserSetting(key, result);
            }
        }

        return result;
    } catch (e) {
        console.error("userSettings set error", e);
    }

    return defaults[key];
};

/**
 * Factory generating a Svelte writable() for the given state type S.
 *
 * The idea is that the initialValue will be the UserSettings value at generation time, and the
 * setter will be a function which invokes setUserSetting() with the updated value.
 *
 * Example for a boolean state value for "followCurrentQueueItem". `followCurrentQueueItem` will
 * be a Svelte writable which can be used in any consuming component.
 *
 * ```typescript
 * const followCurrentQueueItemImpl = persistedUserSettingFactory<boolean>(
 *     await getUserSetting("followCurrentQueueItem"),
 *     async (value: boolean) => {
 *         await setUserSetting("followCurrentQueueItem", value);
 *     }
 * );
 *
 * export let followCurrentQueueItem = await followCurrentQueueItemImpl();
 * ```
 */
export const persistedUserSettingFactory = <S>(
    initialValue: S,
    setter: (value: S) => Promise<void>,
) => {
    return async () => {
        const state = writable<S>(initialValue);

        const customSet = async (value: S) => {
            await setter(value);
            state.set(value);
        };

        return {
            ...state,
            set: customSet,
        };
    };
};
