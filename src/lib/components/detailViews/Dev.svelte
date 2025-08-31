<script lang="ts">
    import { devices, themeManager, webSocketClientStatus, DEV_MODE } from "../../state.ts";
    import { amplifierMuteOff, amplifierMuteOn, amplifierMuteToggle, amplifierPowerOff, amplifierPowerOn, amplifierPowerToggle, amplifierVolumeDown, amplifierVolumeUp, testStreamerConnection } from "../../commands.ts";
    import { getUserSetting, setUserSetting } from "../../userSettings.ts";
    import preAmpManager from "../../preAmpManager.ts";
    import artCache from "../../artCache.ts";
    import pubSub from "../../pubSub.ts";
    import AudioSource from "../dataDisplay/AudioSource.svelte";
    import Loader from "../Loader.svelte";

    const preAmp = preAmpManager();

    let sourceIds = [
        "IR",
        "USB_AUDIO",
        "SPDIF_COAX",
        "SPDIF_TOSLINK",
        "MEDIA_PLAYER",
        "AIRPLAY",
        "SPOTIFY",
        "CAST",
        "ROON",
        "TIDAL"
    ];

    let backgroundColor: string;

    $: if ($themeManager) {
        backgroundColor = $themeManager.getCssVar("--app-background-color");
    }
</script>

{#if DEV_MODE}
    <div class="Dev">
        <div>
            <span>{$preAmp?.volume}</span>
        </div>
        <div>
            <button on:click={amplifierMuteOn}>
                mute on
            </button>
            <button on:click={amplifierMuteOff}>
                mute off
            </button>
            <button on:click={amplifierMuteToggle}>
                mute toggle
            </button>
            <button on:click={amplifierPowerOn}>
                power on
            </button>
            <button on:click={amplifierPowerOff}>
                power off
            </button>
            <button on:click={amplifierPowerToggle}>
                power toggle
            </button>
            <button on:click={amplifierVolumeDown}>
                volume down
            </button>
            <button on:click={amplifierVolumeUp}>
                volume up
            </button>
        </div>
        <div>
            <span>Cache size: </span><span>{`${(artCache.calculatedSize / 1024 / 1024).toFixed(2)}MB`}</span>
        </div>
        <div>
            <button on:click={() => pubSub.publish("AppErrorLog", { level: "error", message: "test error", when: 123})}>
                publish error
            </button>
        </div>
        <div>
            <span>WebSocket Client status:</span>
            <span>{JSON.stringify($webSocketClientStatus)} :: {$devices.is_testing_connection}</span>
            <button on:click={() => testStreamerConnection()}>test connection</button>
        </div>
        <Loader />
        <button on:click={() => setUserSetting("queueDisplay", "detailed")}>Set: detailed</button>
        <button on:click={() => setUserSetting("queueDisplay", "simple")}>Set: simple</button>
        <button on:click={() => getUserSetting("queueDisplay").then((val) => console.log(val))}>Get</button>
        <div style="color: green">{$themeManager.theme}</div>
        <div style="color: red">{backgroundColor}</div>
        <button on:click={() => $themeManager.setTheme("light")}>light</button>
        <button on:click={() => $themeManager.setTheme("dark")}>dark</button>
        <button on:click={() => $themeManager.setTheme("auto")}>auto</button>
        {#each sourceIds as sourceId}
            <AudioSource id={sourceId} />
        {/each}
    </div>
{/if}

<style>
    .Dev {
        display: flex;
        flex-direction: column;
        gap: 5px;
        width: 150px;
    }
</style>
