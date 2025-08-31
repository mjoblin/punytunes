<script lang="ts">
    import { afterUpdate } from "svelte";

    import {
        devices,
        nowPlaying,
        playState,
        positionInternal,
        presets,
        queueList,
        selectedPayload,
        systemInfo,
        systemPowerInternal,
        systemSources,
        zoneState
    } from "../../state.ts";
    import pubSub from "../../pubSub.ts";
    import JsonView from "../dataDisplay/JsonView.svelte";

    afterUpdate(() => {
        pubSub.publish("DetailViewUpdated");
    });

    const payloadChanged = (event: Event) => {
        const target = event.target as HTMLSelectElement;
        selectedPayload.set(target.value);
    };
</script>

<div class="Payloads">
    <div class="payload-select">
        <select bind:value={$selectedPayload} on:change={payloadChanged}>
            <option value="devices">Devices</option>
            <option value="nowPlaying">NowPlaying</option>
            <option value="playState">PlayState</option>
            <option value="positionInternal">Position</option>
            <option value="presets">Presets</option>
            <option value="queueList">QueueList</option>
            <option value="systemInfo">SystemInfo</option>
            <option value="systemPowerInternal">SystemPower</option>
            <option value="systemSources">SystemSources</option>
            <option value="zoneState">ZoneState</option>
        </select>
    </div>

    <div>
        {#if $selectedPayload === "devices"}
            <JsonView data={$devices || {}} />
        {:else if $selectedPayload === "nowPlaying"}
            <JsonView data={$nowPlaying || {}} />
        {:else if $selectedPayload === "playState"}
            <JsonView data={$playState || {}} />
        {:else if $selectedPayload === "positionInternal"}
            <JsonView data={$positionInternal || {}} />
        {:else if $selectedPayload === "presets"}
            <JsonView data={$presets || {}} />
        {:else if $selectedPayload === "queueList"}
            <JsonView data={$queueList || {}} />
        {:else if $selectedPayload === "systemInfo"}
            <JsonView data={$systemInfo || {}} />
        {:else if $selectedPayload === "systemPowerInternal"}
            <JsonView data={$systemPowerInternal || {}} />
        {:else if $selectedPayload === "systemSources"}
            <JsonView data={$systemSources || {}} />
        {:else if $selectedPayload === "zoneState"}
            <JsonView data={$zoneState || {}} />
        {/if}
    </div>
</div>

<style>
    .Payloads {
        display: flex;
        flex-direction: column;
        align-items: stretch;
    }

    .payload-select {
        position: fixed;
        display: flex;
        gap: 5px;
        align-self: flex-end;
    }
</style>