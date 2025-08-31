<script lang="ts">
    import { get } from "svelte/store";
    import { fade } from "svelte/transition";

    import { activeDetailsView, devices, isActivating } from "../../state.ts";
    import { stopWebSocketClient } from "../../commands.ts";
    import Loader from "../Loader.svelte";

    const DELAY_MS = 500;

    let deviceBeingActivated: string | undefined = undefined;
    let display = false;
    let displayTimeout: number;

    $: deviceBeingActivated = $devices?.discovered?.find(device => device.is_activating)?.friendly_name;

    // Don't display anything until isActivating has been true for DELAY_MS.
    $: if ($isActivating) {
        displayTimeout && clearTimeout(displayTimeout);
        displayTimeout = setTimeout(() => {
            if (get(activeDetailsView) !== "streamer") {
                display = true;
            }
        }, DELAY_MS);
    } else {
        displayTimeout && clearTimeout(displayTimeout);
        display = false;
    }
</script>

{#if display}
    <div class="ActivatingDevice" transition:fade={{ duration: 250 }}>
        <div class="message">
            <Loader size={23} />
            <span>{`Connecting${deviceBeingActivated ? ` to ${deviceBeingActivated}` : ""}...`}</span>
        </div>

        <button on:click={() => stopWebSocketClient(true)}>
            cancel
        </button>
    </div>
{/if}

<style>
    .ActivatingDevice {
        position: absolute;
        top: 0;
        left: 0;
        bottom: 0;
        right: 0;
        display: flex;
        flex-direction: column;
        gap: 15px;
        align-items: center;
        justify-content: center;
        background-color: rgb(35, 35, 35, 0.70);
        backdrop-filter: blur(1px);
        -webkit-backdrop-filter: blur(1px);

        @media (prefers-color-scheme: light) {
            background-color: rgb(235, 235, 235, 0.70);
        }

        & > button {
            font-size: 0.8rem;
            padding: 5px 8px;
        }
    }

    .message {
        display: flex;
        gap: 10px;
        color: var(--text-mid);
        font-weight: 600;
        align-items: center;
    }
</style>