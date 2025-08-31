import { get } from "svelte/store";
import {
    checkUpdate,
    installUpdate,
    onUpdaterEvent,
    type UpdateManifest,
} from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";

import { appUpdateState, DEV_MODE } from "./state.ts";

const ONE_DAY_MS = 60 * 60 * 24 * 1000;
const ONE_MINUTE_MS = 60 * 1000;

// ------------------------------------------------------------------------------------------------
// Manage application updates (checking for updates, and performing updates).
//
// We want the app's Svelte state to be the source of truth for app all update information. This
// manager takes care of setting that state (so it can be referenced elsewhere in the app), as well
// as leveraging Tauri's updater functionality to perform updates.
//
// Reference: https://tauri.app/v1/guides/distribution/updater
// ------------------------------------------------------------------------------------------------

/**
 * Check if an app update is available.
 *
 * Tauri calls the endpoint defined under tauri.updater.endpoints in tauri.conf.json.
 */
export const checkIfUpdateAvailable = async () => {
    let isUpdateAvailableCheck = false;
    let manifestCheck: UpdateManifest | undefined = undefined;

    appUpdateState.set({
        checkForUpdateError: undefined,
        isCheckingForUpdate: true,
        isUpdateAvailable: isUpdateAvailableCheck,
        manifest: manifestCheck,
        updateError: undefined,
        updateStatus: undefined,
    });

    let checkError: string | undefined = undefined;

    try {
        const { shouldUpdate, manifest } = !DEV_MODE && await checkUpdate();
        isUpdateAvailableCheck = shouldUpdate;
        manifestCheck = manifest;
    } catch (error) {
        console.error(`Error while checking for application updates: ${error}`);
        checkError = `${error}`;
    }

    appUpdateState.update((state) => ({
        ...state,
        checkForUpdateError: checkError,
        isCheckingForUpdate: false,
        isUpdateAvailable: isUpdateAvailableCheck,
        manifest: manifestCheck,
    }));
};

/**
 * Check for updates once a day while the app is running. We check once per minute whether we're
 * overdue for the daily update check. (The once-per-minute check is done to raise the likelihood
 * of the update check being performed when the host machine is going in and out of sleep; or if
 * the app is otherwise backgrounded/slept in some way.
 */
const startRegularUpdateCheck = async () => {
    await checkIfUpdateAvailable();

    let lastCheckTime = Date.now();

    setInterval(async () => {
        const now = Date.now();

        if (now - lastCheckTime > ONE_DAY_MS) {
            await checkIfUpdateAvailable();
            lastCheckTime = now;
        }
    }, ONE_MINUTE_MS);
};

startRegularUpdateCheck();

/**
 * Perform an application update.
 */
export const doAppUpdate = async () => {
    const unlisten = await onUpdaterEvent(({ error, status }) => {
        appUpdateState.update((state) => ({
            ...state,
            updateError: error,
            updateStatus: status,
        }));
    });

    try {
        if (get(appUpdateState).isUpdateAvailable) {
            // This will also restart the app on Windows
            await installUpdate();
        }
    } catch (error) {
        console.error(`Error installing application update: ${error}`);
    }

    unlisten();
};

/**
 * Relaunch the application.
 */
export const relaunchApp = async () => {
    await relaunch();
};
