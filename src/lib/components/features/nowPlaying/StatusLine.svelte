<script lang="ts">
    import { isHandlingAmplifier, isPowerOn, isInStandby } from "../../../state.ts";
    import AmplifierPowerButton from "../../buttons/AmplifierPowerButton.svelte";
    import AudioFormat from "../../dataDisplay/AudioFormat.svelte";
    import AudioSource from "../../dataDisplay/AudioSource.svelte";
    import ConnectionTestIndicator from "../../dataDisplay/ActivatingIndicator.svelte";
    import PowerButton from "../../buttons/PowerButton.svelte";
    import WarningMessage from "../../dataDisplay/WarningMessage.svelte";
</script>

<div class="StatusLine">
    <div class="lhs">
        {#if $isPowerOn}
            <AudioSource />
            <AudioFormat />
        {/if}
    </div>

    <div class="rhs">
        {#if $isInStandby}
            <WarningMessage>
                streamer is in standby
            </WarningMessage>
        {/if}
        <div class="connection-test-and-power">
            <ConnectionTestIndicator />
            {#if $isHandlingAmplifier}
                <AmplifierPowerButton />
            {/if}
            <PowerButton />
        </div>
    </div>
</div>

<style>
    .StatusLine {
        display: flex;
        font-size: 0.8em;
        justify-content: space-between;
    }

    .lhs {
        display: flex;
        align-items: center;
        gap: 1em;
    }

    .rhs {
        display: flex;
        align-items: center;
        gap: 10px;

        & .connection-test-and-power {
            display: flex;
            gap: 8px;
            align-items: center;
        }
    }
</style>