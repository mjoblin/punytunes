import { invoke } from "@tauri-apps/api/tauri";

import type { AmplifierAction } from "../types/generated/AmplifierAction.ts";
import type { StreamMagicDevice } from "../types/generated/StreamMagicDevice.ts";
import type { StreamerAction } from "../types/generated/StreamerAction.ts";
import type { TransportToggleState } from "../types/generated/streammagic_payloads/TransportToggleState.ts";
import { isConnectionTestPending } from "./state.ts";

// ------------------------------------------------------------------------------------------------
// Tauri Rust commands

export const activateDevice = async (device: StreamMagicDevice) => {
    await invoke("activate_device", { udn: device.udn });
}

export const deactivate = async () => {
    await invoke("deactivate");
}

export const discoverAmplifier = async () => {
    await invoke("discover_amplifier");
}

export const discoverStreamer = async () => {
    await invoke("discover_streamer");
}

export const emitAppLog = async (level: string, message: string) => {
    await invoke("emit_app_log", { level, message });
}

export const sendAmplifierAction = async (action: AmplifierAction) => {
    await invoke("send_amplifier_action", { action });
}

export const sendStreamerAction = async (action: StreamerAction) => {
    await invoke("send_streamer_action", { action });
}

export const shutdown = async () => {
    await invoke("shutdown");
}

export const stopWebSocketClient = async (deleteFromPersistedState: boolean = false) => {
    await invoke("stop_websocket_client", { deleteFromPersistedState });
}

export const testAmplifierConnection = async () => {
    await invoke("test_amplifier_connection");
}

export const testStreamerConnection = async () => {
    isConnectionTestPending.set(true);
    await invoke("test_streamer_connection");
}

export const uiReady = async () => {
    await invoke("ui_ready");
}

// ------------------------------------------------------------------------------------------------
// Streamer action helpers for the sendStreamerAction Tauri Rust command

export const muteOn = async() => {
    await sendStreamerAction({ "MuteSet": true });
}

export const muteOff = async() => {
    await sendStreamerAction({ "MuteSet": false });
}

export const nextTrack = async() => {
    await sendStreamerAction("NextTrack");
}

export const pause = async() => {
    await sendStreamerAction("Pause");
}

export const play = async() => {
    await sendStreamerAction("Play");
}

export const playQueueId = async(queueId: number) => {
    await sendStreamerAction({ "PlayQueueId": queueId });
}

export const playPresetId = async(presetId: number) => {
    await sendStreamerAction({ "PlayPresetId": presetId });
}

export const powerOn = async() => {
    await sendStreamerAction("PowerOn");
}

export const powerStandby = async() => {
    await sendStreamerAction("PowerStandby");
}

export const powerToggle = async() => {
    await sendStreamerAction("PowerToggle");
}

export const previousTrack = async() => {
    await sendStreamerAction("PreviousTrack");
}

export const seek = async(position: number) => {
    await sendStreamerAction({ "Seek": position });
}

export const setRepeatState = async(state: TransportToggleState) => {
    await sendStreamerAction({ "SetRepeat": state });
}

export const setShuffleState = async(state: TransportToggleState) => {
    await sendStreamerAction({ "SetShuffle": state });
}

export const setSourceId = async(sourceId: string) => {
    await sendStreamerAction({ "SetSourceId": sourceId });
}

export const stop = async() => {
    await sendStreamerAction("Stop");
}

export const togglePlayback = async() => {
    await sendStreamerAction("TogglePlayback");
}

export const volumePercentSet = async(percent: number) => {
    await sendStreamerAction({ "VolumePercentSet": percent });
}

export const volumeStepChange = async(changeAmount: number) => {
    await sendStreamerAction({ "VolumeStepChange": changeAmount });
}

export const volumeStepSet = async(step: number) => {
    await sendStreamerAction({ "VolumeStepSet": step });
}

// ------------------------------------------------------------------------------------------------
// Amplifier action helpers for the sendAmplifierAction Tauri Rust command

export const amplifierMuteOn = async() => {
    await sendAmplifierAction({ "MuteSet": true });
}

export const amplifierMuteOff = async() => {
    await sendAmplifierAction({ "MuteSet": false });
}

export const amplifierMuteToggle = async() => {
    await sendAmplifierAction("MuteToggle");
}

export const amplifierPowerOn = async() => {
    await sendAmplifierAction({ "PowerSet": true });
}

export const amplifierPowerOff = async() => {
    await sendAmplifierAction({ "PowerSet": false });
}

export const amplifierPowerToggle = async() => {
    await sendAmplifierAction("PowerToggle");
}

export const amplifierSourceSet = async(sourceId: number) => {
    await sendAmplifierAction({ "SourceSet": sourceId });
}

export const amplifierVolumeDown = async() => {
    await sendAmplifierAction("VolumeDown");
}

export const amplifierVolumeSet = async(level: number) => {
    await sendAmplifierAction({ "VolumeSet": level });
}

export const amplifierVolumeUp = async() => {
    await sendAmplifierAction("VolumeUp");
}
