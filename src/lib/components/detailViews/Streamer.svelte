<script lang="ts">
    import { afterUpdate } from "svelte";
    import {
        IconPlug,
        IconPlugOff,
        IconSettings,
        IconWifi,
        IconWifi0,
        IconWifi1,
        IconWifi2
    } from "@tabler/icons-svelte";
    import { open } from "@tauri-apps/api/shell";

    import type { StreamMagicDevice } from "../../../types/generated/StreamMagicDevice.ts";
    import { activateDevice, deactivate, discoverAmplifier, discoverStreamer } from "../../commands.ts";
    import { amplifierManagerState, devices, themeManager, webSocketClientStatus } from "../../state.ts";
    import pubSub from "../../pubSub.ts";
    import IconButton from "../buttons/IconButton.svelte";
    import Tooltip from "../overlays/Tooltip.svelte";

    $: haveStreamers = $devices.discovered.length > 0;
    $: activeDevice = $devices.discovered.find((device) => device.is_active);
    $: activatingDevice = $devices.discovered.find((device) => device.is_activating);

    $: managedAmplifier = $amplifierManagerState?.managed_device;

    const accentColor = "--accent-color";
    const accentColorBrighter = "--accent-color-brighter";
    const accentColorBrightest = "--accent-color-brightest";
    const findButtonColor = $themeManager.theme === "light" ? accentColorBrighter : "--text-normal";
    const streamerActionButtonColor = $themeManager.theme === "light" ? accentColorBrighter : accentColorBrightest;
    const streamerActionButtonColorFilled = $themeManager.theme === "light" ? accentColorBrighter : accentColor;

    afterUpdate(() => {
        pubSub.publish("DetailViewUpdated");
    });

    // Keep track of when discovery is happening. Once discovery completes, auto-activate the
    // first streamer if none are already active.

    let discovering = false;

    $: if (!!$devices?.is_discovering) {
        discovering = true;
    } else if (!$devices?.is_discovering && discovering) {
        discovering = false;

        if ($devices.discovered.length > 0 && $webSocketClientStatus.state !== "Connected") {
            activateDevice($devices.discovered[0]);
        }
    }

    // Oscillate through WiFi icons when performing discovery -------------------------------------

    const isDiscoveringIcons = [IconWifi0, IconWifi1, IconWifi2, IconWifi];
    let isDiscoveringIcon = isDiscoveringIcons.at(-1);

    $: !$devices.is_discovering && stopIteratingOverDiscoveringIcons();

    let isDiscoveringIconIndex = 0;
    let isDiscoveringIconDirection = 1;
    let isDiscoveringInterval: number | undefined;
    let delayingResults = false;

    // Contrive a delay before showing the first result. The first result often comes in
    // immediately, so without a delay it's not clear that a search has actually taken place.
    const startDelayedResultsTimer = () => {
        delayingResults = true;
        setTimeout(() => delayingResults = false, 2000);
    };

    const iterateOverDiscoveringIcons = () => {
        isDiscoveringInterval = setInterval(() => {
            isDiscoveringIcon = isDiscoveringIcons[isDiscoveringIconIndex];
            isDiscoveringIconIndex += isDiscoveringIconDirection;

            if (isDiscoveringIconIndex === isDiscoveringIcons.length - 1 || isDiscoveringIconIndex === 0) {
                isDiscoveringIconDirection *= -1;
            }
        }, 300);
    };

    const stopIteratingOverDiscoveringIcons = () => {
        isDiscoveringIcon = isDiscoveringIcons.at(-1);
        isDiscoveringInterval && clearInterval(isDiscoveringInterval);
        isDiscoveringInterval = undefined;
    };

    // Helper functions ---------------------------------------------------------------------------

    const deviceIp = (device: StreamMagicDevice): string => {
        const urlParsed = new URL(device.url);
        return urlParsed.hostname;
    };

    const openStreamerConfig = (device: StreamMagicDevice) => {
        open(`http://${deviceIp(device)}`);
    };
</script>

<div class="Streamer">
    <div class="streamer-info-header">
        {#if haveStreamers && !delayingResults}
            <span>Streamers found:</span>
        {:else}
            <span />
        {/if}
        <div>
            <IconButton
                icon={isDiscoveringIcon}
                variant={$themeManager.theme === "light" ? "outline" : "subtle"}
                color={findButtonColor}
                disabled={$devices.is_discovering}
                on:click={() => {
                    startDelayedResultsTimer();
                    discoverStreamer();
                    discoverAmplifier();
                    iterateOverDiscoveringIcons();
                }}
            >
                Find
            </IconButton>
        </div>
    </div>

    <div class="streamer-info-body">
        {#if haveStreamers && !delayingResults}
            <div class="streamers-available">
                <table>
                    {#each $devices.discovered as device}
                        <tr>
                            <td class="streamer-name">{device.friendly_name}</td>
                            <td>{device.model}</td>
                            <td class="streamer-ip">{deviceIp(device)}</td>
                            <td>
                                {#if device.is_activating || device.is_active}
                                    <div class="connection-state-container">
                                        <Tooltip
                                            label={device.is_activating ? "connecting" : "connected"}
                                            offset={16}
                                        >
                                            <div
                                                class="connection-state"
                                                class:connecting={device.is_activating}
                                                class:connected={device.is_active}
                                            />
                                        </Tooltip>
                                    </div>
                                {:else}
                                    <div class="connection-state-container">
                                        <div class="connection-state" />
                                    </div>
                                {/if}
                            </td>
                            <td class="streamer-actions">
                                <div class="streamer-actions-container">
                                    {#if device.is_active}
                                        <IconButton
                                            icon={IconPlugOff}
                                            variant="outline"
                                            color={streamerActionButtonColor}
                                            size={12}
                                            on:click={() => deactivate()}
                                        >
                                            disconnect
                                        </IconButton>
                                    {:else}
                                        <IconButton
                                            icon={IconPlug}
                                            variant="filled"
                                            color={streamerActionButtonColorFilled}
                                            size={12}
                                            on:click={() => activateDevice(device)}
                                        >
                                            connect
                                        </IconButton>
                                    {/if}

                                    <Tooltip label="view settings">
                                        <IconButton
                                            icon={IconSettings}
                                            variant="outline"
                                            color={streamerActionButtonColor}
                                            size={12}
                                            on:click={() => openStreamerConfig(device)}
                                        />
                                    </Tooltip>
                                </div>
                            </td>
                        </tr>
                    {/each}
                </table>

                <div class="connected-devices">
                    <div class="connected-device">
                        {#if activeDevice}
                            Connected to <b>{activeDevice.friendly_name}</b> ({activeDevice.model})
                        {:else if activatingDevice}
                            Connecting to <b>{activatingDevice.friendly_name}</b>...
                        {:else}
                            Not currently connected to an available streamer.
                        {/if}
                    </div>

                    {#if $amplifierManagerState?.is_handling_amplifier && managedAmplifier}
                        <div class="connected-device">
                            Connected to amplifier <b>{managedAmplifier.friendly_name}</b>
                            ({`${managedAmplifier.manufacturer} ${managedAmplifier.model}`})
                        </div>
                    {/if}
                </div>
            </div>
        {:else if $devices.is_discovering}
            <div class="detail-data-empty no-streamers">
                <div>
                    Looking for StreamMagic streamers...
                </div>
            </div>
        {:else if !$devices.is_discovering}
            <div class="detail-data-empty no-streamers">
                <div>
                    <b>No streamers found.</b>
                </div>
                <div>
                    PunyTunes uses UPnP Discovery to find StreamMagic streamers on your network. Click
                    "Find" to try again.
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .Streamer {
        display: flex;
        flex-direction: column;
        gap: 10px;
        padding: 5px;

        & table {
            background-color: var(--background-dim);
        }

        & tr {
            vertical-align: middle;
            cursor: default;
        }
    }

    .streamer-info-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .streamer-info-body {
        min-height: 60px;
    }

    .streamers-available {
        display: flex;
        flex-direction: column;
        gap: 15px;
    }

    .no-streamers {
        display: flex;
        flex-direction: column;
        gap: 10px;
        text-align: center;
        padding-top: 0;
    }

    .streamer-name {
        font-weight: bold;
        word-break: break-all;
    }

    .streamer-ip {
        font-size: 0.9em;
    }

    .connection-state-container {
        display: flex;
        justify-content: center;

        & .connection-state {
            width: 10px;
            height: 10px;
            border-radius: 5px;

            &.connecting {
                background: yellow;
            }

            &.connected {
                background: limegreen;

                @media (prefers-color-scheme: light) {
                    background-color: var(--success-color);
                }
            }
        }
    }

    .streamer-actions {
        text-align: right;
    }

    .streamer-actions-container {
        display: inline-flex;
        gap: 5px;
        justify-content: flex-end;
        min-width: 115px;
    }

    .connected-devices {
        display: flex;
        flex-direction: column;
        gap: 3px;
    }

    .connected-device {
        color: var(--text-mid);
        word-break: break-all;
    }
</style>