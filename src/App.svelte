<script lang="ts">
    import { afterUpdate, onDestroy, onMount } from "svelte";
    import { appWindow, LogicalSize } from "@tauri-apps/api/window";

    import {
        detailsPosition,
        devices,
        DEV_MODE,
        isConnected,
        isEstablishingConnectionState,
        isHandlingAmplifier,
        osType,
        uiInitialized,
        wasConnectedAtAppHide,
        webSocketClientStatus,
    } from "./lib/state.ts";
    import { activateDevice, testAmplifierConnection, testStreamerConnection } from "./lib/commands.ts";
    import pubSub from "./lib/pubSub.ts";
    import ActivatingDevice from "./lib/components/overlays/ActivatingDevice.svelte";
    import Details from "./lib/components/Details.svelte";
    import KeyboardShortcuts from "./lib/components/KeyboardShortcuts.svelte";
    import LatestError from "./lib/components/overlays/LatestError.svelte";
    import NowPlaying from "./lib/components/NowPlaying.svelte";

    let thisElement: HTMLElement;
    let topLevelAppElement = document.getElementById("app");
    let windowWidth = window.innerWidth;
    let isAppResizableAtStartup = false;
    let firstUpdateComplete = false;
    let firstTrayClick = true;
    let lastLostFocus = Date.now();

    if (!DEV_MODE) {
        // Prevent right-clicking.
        document.addEventListener("contextmenu", e => {
            e.preventDefault();
            return false;
        }, { capture: true });

        document.addEventListener("selectstart", e => {
            e.preventDefault();
            return false;
        }, { capture: true });
    }

    // Once the discovered device list arrives and discovery has completed, auto-connect to the
    // first one if we're not already connected to a streamer.
    let haveAutoConnectedAtStartup = false;

    $: if (!haveAutoConnectedAtStartup && $devices.discovered.length > 0) {
        if (!$devices.is_discovering) {
            if ($webSocketClientStatus.state !== "Connected") {
                activateDevice($devices.discovered[0]);
            }

            haveAutoConnectedAtStartup = true;
        }
    }

    // Unsubscribe functions to be invoked on destroy.
    let unsubscribeFns: (() => void)[] = [];

    onMount(async () => {
        // The app shouldn't be resizable unless running with the FLOATING tauri.conf.
        isAppResizableAtStartup = await appWindow.isResizable();
    });

    afterUpdate(async () => {
        firstUpdateComplete = true;
        await resizeAppWindow();
    });

    onDestroy(() => {
        unsubscribeFns.forEach((unsubscribeFn) => {
            unsubscribeFn();
        });
    });

    // Whenever the app updates (usually because a new detail screen was activated), set the
    // application window height to match the currently-displayed content. This ensures that the
    // app window is never taller than the content it's displaying, which in turn ensures that
    // clicking underneath the window will be treated as an "outside click" (which is not the case
    // if the window is always set to the tallest-allowed height regardless of content shown).
    //
    // Note: The window can still never exceed the window height set in tauri.conf.json.
    // Note: The "+ 2" seems to be required for the window decoration border to not be cropped.
    // Note: The window width should never change in normal operation.

    let resizeAppWindow = async () => {
    };

    $: resizeAppWindow = async () => {
        if ($isEstablishingConnectionState) {
            return;
        }

        if (!firstUpdateComplete) {
            // If we don't bail out of here before the first call to afterUpdate() has been made,
            // then the app can get into a weird state at startup if the Queue detail view is
            // active. The UI seems to get offset vertically in a weird way, where mouse clicks
            // seem to be acting as if the view is actually 20-30 pixels below where it's being
            // rendered. Guess: This might be due to AppResizeComplete firing too early, which
            // can result in the Queue auto-scroll happening, which is maybe throwing things off.
            return;
        }

        if (!isAppResizableAtStartup) {
            // Windows doesn't behave as we want when calling setSize(). Windows will maintain the
            // position of the top of the application window and adjust the bottom of the window,
            // whereas we want the window to be locked to the task bar at the bottom and grow
            // upwards.
            if ($osType !== "Windows_NT") {
                // Setting the size will adjust the app window to match the current HTML DOM
                // element. This ensures that clicks outside the app are properly detected, which
                // in turn ensures the app window will close on an outside click.
                const newAppSize = new LogicalSize(windowWidth || 450, thisElement.offsetHeight + 2);
                await appWindow.setSize(newAppSize);
            }
        }

        pubSub.publish("AppResizeComplete");
    };

    // Whenever a detail view updates, resize the app window to ensure it's tall enough to contain
    // the entire content.
    unsubscribeFns.push(
        pubSub.subscribe("DetailViewUpdated", () => {
            resizeAppWindow();
        })
    );

    // Show the application.
    const showApp = async () => {
        pubSub.publish("TrayAppOpened");

        await appWindow.show();
        await appWindow.setFocus();

        if ($uiInitialized && $wasConnectedAtAppHide && $webSocketClientStatus.state !== "Connecting") {
            await testStreamerConnection();

            if ($isHandlingAmplifier) {
                await testAmplifierConnection();
            }
        }
    };

    // Hide the application.
    const hideApp = async () => {
        await appWindow.hide();
        wasConnectedAtAppHide.set(!!$isConnected);

        pubSub.publish("TrayAppClosed");
    };

    // When the application's system tray icon is clicked, decide whether to show or hide the
    // application.
    //
    // Note: Whenever the user clicks on the tray icon, the application will receive both a
    //  TrayLeftClick *and* an AppLostFocus. (The system tray icon is not considered part of the
    //  app window, so clicking it is a focus loss). The order of these two events is different
    //  on macOS (TrayLeftClick then AppLostFocus) and Windows (AppLostFocus then TrayLeftClick),
    //  so we special-case Windows to ignore the TrayLeftClick if it arrives 'immediately' after
    //  an AppLostFocus. If we don't do that, then the app would just be displayed again when
    //  the user's intent was for the app to be hidden.
    unsubscribeFns.push(
        pubSub.subscribe("TrayLeftClick", () => {
            const handleTrayLeftClick = async() => {
                const isWindowVisible = await appWindow.isVisible();

                if (firstTrayClick) {
                    await showApp();
                    firstTrayClick = false;
                } else if (isWindowVisible) {
                    await hideApp();
                } else {
                    if ($osType === "Windows_NT") {
                        const now = Date.now();
                        const WINDOWS_IGNORE_LOST_FOCUS_DURATION = 250;

                        if (firstTrayClick || (now - lastLostFocus >= WINDOWS_IGNORE_LOST_FOCUS_DURATION)) {
                            await showApp();
                        }
                    } else {
                        await showApp();
                    }
                }
            };

            handleTrayLeftClick();
        })
    );

    // When the app loses focus, hide the application.
    unsubscribeFns.push(
        pubSub.subscribe("AppLostFocus", () => {
            const handleAppLostFocus = async () => {
                await hideApp();
                lastLostFocus = Date.now();
            };

            handleAppLostFocus();
        })
    );
</script>

<svelte:window on:click={(event) => {
    if ($osType === "Windows_NT" && event.target === topLevelAppElement && event.offsetY >= event.pageY) {
        // On Windows, we're not calling appWindow.setSize(), so we have to manually detect when
        // the user clicks above the visible application area but below the Tauri application
        // window. When that happens, we want to do the same thing as an outside-click as otherwise
        // detected on the Rust side (look for "tauri::WindowEvent::Focused" in main.rs).
        hideApp();
    }
}} />

<main
    class="App"
    class:details-on-bottom={$detailsPosition === "bottom"}
    class:details-on-top={$detailsPosition === "top"}
    class:lock-to-top={$osType === "Darwin"}
    class:lock-to-bottom={$osType !== "Darwin"}
    bind:this={thisElement}
>
    <KeyboardShortcuts />
    {#if $detailsPosition === "top"}
        <Details />
        <NowPlaying />
    {:else}
        <NowPlaying />
        <Details />
    {/if}
    <LatestError />
    <ActivatingDevice />
</main>

<style>
    .App {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: stretch;
        gap: 9px;
        border-radius: 7px;
        border: 1px solid var(--background-bright);
        padding-top: 0;
        padding-right: 12px;
        padding-bottom: 0;
        padding-left: 12px;
        color: var(--root-color);
        background-color: var(--app-background-color);
        overflow: hidden;

        @media (prefers-color-scheme: light) {
            border: 1px solid var(--background-mid);
        }

        &.details-on-bottom {
            padding-top: 12px;
        }

        &.details-on-top {
            padding-top: 8px;
            padding-bottom: 12px;
        }

        &.lock-to-top {
            margin-top: 1px;
            margin-bottom: auto;
        }

        &.lock-to-bottom {
            margin-top: auto;
            margin-bottom: 12px;
        }
    }
</style>