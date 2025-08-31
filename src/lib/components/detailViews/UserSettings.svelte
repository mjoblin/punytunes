<script lang="ts">
    import { onMount } from "svelte";
    import { IconInfoCircle } from "@tabler/icons-svelte";
    import {
        disable as disableLaunchAtStartup,
        enable as enableLaunchAtStartup,
        isEnabled as isLaunchAtStartupEnabledCheck
    } from "tauri-plugin-autostart-api";

    import { currentInfoView, detailsPosition, osType, showVolumeControls, themeManager } from "../../state.ts";
    import pubSub from "../../pubSub.ts";
    import IconButton from "../buttons/IconButton.svelte";

    let isLaunchAtStartupEnabled: boolean = false;

    onMount(async () => {
        isLaunchAtStartupEnabled = await isLaunchAtStartupEnabledCheck();
    });

    const handleLaunchAtStartupChange = async (e: Event) => {
        const target = e.target as HTMLInputElement;

        try {
            if (target.checked && !isLaunchAtStartupEnabled) {
                await enableLaunchAtStartup();
            } else if (isLaunchAtStartupEnabled) {
                await disableLaunchAtStartup();
            }

            isLaunchAtStartupEnabled = await isLaunchAtStartupEnabledCheck();
        } catch (e) {
            console.error("launchAtStartup error: ", e);
        }
    };
</script>

<!-- ------------------------------------------------------------------------------------------ -->

<div class="UserSettings">
    <div class="info-button">
        <IconButton
            icon={IconInfoCircle}
            variant="outline"
            color="--text-dim"
            size={14}
            fontSize={11}
            padding="0.6em 0.6em"
            on:click={() => currentInfoView.set("info")}
        >
            App Info
        </IconButton>
    </div>

    <div class="settings-choices">
        <div class="title">Settings</div>

        <!-- Theme -->
        <div class="setting-theme-container">
            <div class="setting-theme">
                <span>Theme:</span>
                <div class="theme-choices">
                    <button
                        class:active={$themeManager.userTheme === "light"}
                        class:no-right-border={$themeManager.userTheme === "auto"}
                        on:click={() => $themeManager.setTheme("light")}
                    >
                        Light
                    </button>

                    <button
                        class:active={$themeManager.userTheme === "auto"}
                        on:click={() => $themeManager.setTheme("auto")}
                    >
                        Auto
                    </button>

                    <button
                        class:active={$themeManager.userTheme === "dark"}
                        class:no-left-border={$themeManager.userTheme === "auto"}
                        on:click={() => $themeManager.setTheme("dark")}
                    >
                        Dark
                    </button>
                </div>
            </div>
            {#if $osType === "Windows_NT"}
                <div class="setting-explanation">
                    Changing the theme will restart the application in Windows.
                </div>
            {/if}
        </div>

        <!-- Launch at startup -->
        <div class="setting-launch-container">
            <span>Launch at startup:</span>
            <input type="checkbox" checked={isLaunchAtStartupEnabled} on:change={handleLaunchAtStartupChange}>
        </div>

        <!-- Whether to show the volume controls -->
        <div class="setting-volume-controls-container">
            <div class="setting-volume-controls">
                <span>Show volume controls:</span>
                <input
                    type="checkbox"
                    checked={$showVolumeControls}
                    on:change={() => showVolumeControls.set(!$showVolumeControls)}
                >
            </div>

            <div class="setting-explanation">
                Volume controls will only be visible <b>when the streamer's Pre-Amp mode is
                enabled</b>, or if a Hegel amplifier is found on the network.
            </div>
        </div>

        <!-- Where to display the Details screens -->
        <div class="setting-details-position-container">
            <div class="setting-details-position">
                <span>Details position:</span>
                <div class="details-position-choices">
                    <button
                        class:active={$detailsPosition === "bottom"}
                        on:click={() => {
                            detailsPosition.set("bottom");
                            pubSub.publish("DetailViewUpdated");
                        }}
                    >
                        Below
                    </button>

                    <button
                        class:active={$detailsPosition === "top"}
                        on:click={() => {
                            detailsPosition.set("top");
                            pubSub.publish("DetailViewUpdated");
                        }}
                    >
                        Above
                    </button>
                </div>
            </div>

            <div class="setting-explanation">
                <b>Changing the details position can be jarring</b>, depending on the location of
                your system tray. This setting moves the details screens (Queue, Presets, etc,
                <b>including these Settings</b>) either <b>below</b> or <b>above</b> the current
                track information.
            </div>
        </div>
    </div>
</div>

<!-- ------------------------------------------------------------------------------------------ -->

<style>
    .UserSettings {
        position: relative;
        display: flex;
        flex-direction: column;
        padding: 5px;
        font-size: 0.9em;
    }

    .info-button {
        position: absolute;
        top: 5px;
        right: 5px;
    }

    .settings-choices {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .title {
        font-size: 1.2em;
        font-weight: 600;
        padding-bottom: 3px;
    }

    .setting-explanation {
        color: var(--text-dim);
        padding-left: 9px;
    }

    .setting-theme-container {
        display: flex;
        flex-direction: column;
        gap: 5px;

        & .setting-theme {
            display: flex;
            gap: 5px;
            align-items: center;

            & .theme-choices {
                display: flex;
                gap: 0;

                & button {
                    color: var(--text-normal);
                }

                & button:not(.active) {
                    background-color: transparent;
                    border: 1px solid var(--background-bright);
                }

                & button.active {
                    border: 1px solid var(--accent-color);

                    @media (prefers-color-scheme: light) {
                        color: var(--text-min);
                    }
                }

                & button:first-of-type {
                    border-top-right-radius: 0;
                    border-bottom-right-radius: 0;
                }

                & button:nth-of-type(2) {
                    border-radius: 0;
                    border-left: none;
                    border-right: none;
                }

                & button:last-of-type {
                    border-top-left-radius: 0;
                    border-bottom-left-radius: 0;
                }

                & button.no-left-border {
                    border-left: 1px solid transparent;
                }

                & button.no-right-border {
                    border-right: 1px solid transparent;
                }
            }
        }
    }

    .setting-launch-container {
        display: flex;
        gap: 5px;
        align-items: center;

        & input {
            width: 15px;
            height: 15px;
        }
    }

    .setting-volume-controls-container {
        display: flex;
        flex-direction: column;
        gap: 3px;

        & .setting-volume-controls {
            display: flex;
            gap: 5px;
            align-items: center;

            & input {
                width: 15px;
                height: 15px;
            }
        }
    }

    .setting-details-position-container {
        display: flex;
        flex-direction: column;
        gap: 5px;

        & .setting-details-position {
            display: flex;
            gap: 5px;
            align-items: center;

            & .details-position-choices {
                display: flex;
                gap: 0;

                & button {
                    color: var(--text-normal);
                }

                & button:not(.active) {
                    background-color: transparent;
                    border: 1px solid var(--background-bright);
                }

                & button.active {
                    border: 1px solid var(--accent-color);

                    @media (prefers-color-scheme: light) {
                        color: var(--text-min);
                    }
                }

                & button:first-of-type {
                    border-top-right-radius: 0;
                    border-bottom-right-radius: 0;
                    border-right: none;
                }

                & button:last-of-type {
                    border-top-left-radius: 0;
                    border-bottom-left-radius: 0;
                    border-left: none;
                }
            }
        }
    }
</style>
