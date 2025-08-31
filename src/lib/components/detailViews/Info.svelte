<script lang="ts">
    import { afterUpdate, onMount } from "svelte";
    import { IconHeart, IconHeartFilled, IconSettings } from "@tabler/icons-svelte";
    import { getVersion } from "@tauri-apps/api/app";
    import { open } from "@tauri-apps/api/shell";

    import { appUpdateState, currentInfoView, themeManager } from "../../state.ts";
    import { checkIfUpdateAvailable, doAppUpdate, relaunchApp } from "../../updatesManager.ts";
    import pubSub from "../../pubSub.ts";
    import IconButton from "../buttons/IconButton.svelte";
    import Loader from "../Loader.svelte";
    import Tooltip from "../overlays/Tooltip.svelte";
    import ErrorMessage from "../dataDisplay/ErrorMessage.svelte";
    import SuccessMessage from "../dataDisplay/SuccessMessage.svelte";
    import WarningMessage from "../dataDisplay/WarningMessage.svelte";

    // Check for updates whenever the Info screen is opened
    onMount(async () => await checkIfUpdateAvailable());

    afterUpdate(() => {
        // This is a Svelte update, not to be confused with an application update.
        pubSub.publish("DetailViewUpdated");
    });

    const copyrightStart = 2024;
    const copyrightEnd = new Date().getFullYear();
    const copyrightDates = `${copyrightStart}${copyrightEnd > copyrightStart ? `-${copyrightEnd}` : ""}`;

    const updateApp = async () => {
        await doAppUpdate();
    };

    const restartApp = async () => {
        await relaunchApp();
    };

    $: updateStatus = $appUpdateState.updateStatus;
</script>

<!-- ------------------------------------------------------------------------------------------ -->

<div class="Info">
    <div class="settings-button">
        <IconButton
            icon={IconSettings}
            variant="outline"
            color="--text-dim"
            size={14}
            fontSize={11}
            padding="0.6em 0.6em"
            on:click={() => currentInfoView.set("userSettings")}
        >
            Settings
        </IconButton>
    </div>

    <!-- App update available -->

    {#if $appUpdateState.isUpdateAvailable}
        <div class="update-available">
            {#if $appUpdateState.updateStatus === "DONE"}
                <SuccessMessage>Update Installed</SuccessMessage>
            {:else}
                <WarningMessage>Update Available</WarningMessage>
            {/if}

            <div class="update-status">
                {#if updateStatus === "PENDING"}
                    <span>Downloading v{$appUpdateState.manifest?.version}...</span>
                {:else if updateStatus === "DOWNLOADED"}
                    <span>Installing v{$appUpdateState.manifest?.version}...</span>
                {:else if updateStatus === "ERROR"}
                    <div class="error">
                        <header>Error installing v{$appUpdateState.manifest?.version}</header>
                        <section>
                            {#each ($appUpdateState.updateError?.replace("Tauri API error", "").split(": ") || []) as errorLine}
                                <div>{errorLine}</div>
                            {/each}
                        </section>
                    </div>
                {:else if updateStatus === "DONE"}
                    <span>v{$appUpdateState.manifest?.version} was successfully installed</span>
                {:else}
                    <span>v{$appUpdateState.manifest?.version} is ready to be installed</span>
                {/if}
            </div>

            <button
                disabled={updateStatus && ["DOWNLOADED", "ERROR", "PENDING"].includes(updateStatus)}
                on:click={updateStatus === "DONE" ? restartApp : updateApp}
            >
                {$appUpdateState.updateStatus === "DONE" ? "Restart PunyTunes" : "Install Update"}
            </button>
        </div>
    {/if}

    <!-- App information -->

    <div>
        <div class="app-name">
            <span><b>PunyTunes</b></span>
            {#await getVersion() then version}
                <span class="version">v{version}</span>
            {/await}
        </div>
        <div class="url release-notes">
            <a href="https://punytunes.app/release-notes" target="_blank">release notes</a>
        </div>
    </div>

    {#if $appUpdateState.isCheckingForUpdate}
        <div class="update-check">
            <Loader size={15} />
            <span>Checking for updates...</span>
        </div>
    {:else if $appUpdateState.checkForUpdateError}
        <div class="update-check">
            <ErrorMessage withIcon={false}>
                {`Problem checking for updates: ${$appUpdateState.checkForUpdateError}`}
            </ErrorMessage>
        </div>
    {:else if !$appUpdateState.isUpdateAvailable}
        <div class="update-check">
            <SuccessMessage>PunyTunes is up to date</SuccessMessage>
        </div>
    {/if}

    <div class="description">
        PunyTunes is a petite controller for StreamMagic music streamers.
    </div>

    <div class="support">
        <span>Support PunyTunes</span>
        <Tooltip label="visit support page" offset={10}>
            <IconButton
                icon={$themeManager.theme === "light" ? IconHeartFilled : IconHeart}
                color={$themeManager.theme === "light" ? "--attention-color" : "--attention-color-bright"}
                size={14}
                fontSize={11}
                padding="0.5em 0.6em"
                variant="outline"
                on:click={() => open("https://punytunes.app/support")}
            >
                {#if $themeManager.theme === "light"}
                    <span style="font-weight: bold">Support</span>
                {:else}
                    Support
                {/if}
            </IconButton>
        </Tooltip>
    </div>

    <div class="fine-print">
        <div class="copyright">
            <span>© <span class="copyright-date">{copyrightDates}</span>
                <a href="https://redactedcat.com" target="_blank">redacted cat</a>
            </span>
        </div>

        <div>•</div>

        <div class="url">
            <a href="https://punytunes.app" target="_blank">https://punytunes.app</a>
        </div>
    </div>
</div>

<!-- ------------------------------------------------------------------------------------------ -->

<style>
    .Info {
        position: relative;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        color: var(--text-normal);
        padding: 15px 20px 20px 20px;
        gap: 10px;
        text-align: center;

        & a {
            color: var(--text-mid);
            text-decoration: none;
        }
    }

    .settings-button {
        position: absolute;
        top: 5px;
        right: 5px;
    }

    .app-name {
        padding-top: 5px;
        font-size: 1.1em;
    }

    .release-notes {
        padding-top: 3px;
    }

    .version {
        font-size: 0.8em;
    }

    .update-available {
        display: flex;
        flex-direction: column;
        gap: 8px;
        width: 100%;
        align-items: center;
        border: 1.5px solid var(--attention-color);
        border-radius: 3px;
        margin-top: 28px;
        padding: 10px 13px 10px 13px;
        background-color: var(--background-dim);

        & .update-status {
            display: flex;
            gap: 5px;

            & .error {
                display: flex;
                flex-direction: column;
                gap: 5px;
                font-weight: 500;

                & header {
                    color: var(--alert-color);
                    font-weight: 600;
                }

                & section {
                    & div {
                        padding: 1px;
                        word-break: break-all;
                    }

                    & div:nth-child(even) {
                        color: var(--text-normal);
                    }

                    & div:nth-child(odd) {
                        color: var(--text-mid);
                    }
                }
            }
        }

        & button {
            font-size: 90%;
        }
    }

    .update-check {
        min-height: 1.8em;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .support {
        display: flex;
        gap: 15px;
        padding-top: 5px;
        padding-bottom: 5px;
        align-items: center;
        justify-content: center;
        color: var(--attention-color);

        @media (prefers-color-scheme: light) {
            font-weight: bold;
        }
    }

    .fine-print {
        display: flex;
        gap: 10px;
        align-items: center;
        color: var(--text-mid);

        & .copyright {
            font-size: 0.9em;

            & .copyright-date {
                font-size: 90%;
            }
        }

        & .url {
            font-size: 0.9em;
        }
    }
</style>
