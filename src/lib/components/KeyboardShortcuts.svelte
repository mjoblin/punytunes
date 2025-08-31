<!--
    Application keyboard shortcuts.

    Application controls:
      C                   - Scroll to current Queue item

    Details toggles:
      Q                   - Toggle Queue details view
      P                   - Toggle Presets details view
      S                   - Toggle Source details view
      T                   - Toggle Streamer details view
      I                   - Toggle Info details view

    Pre-amp:
      M                   - Mute toggle
      ArrowUp             - Volume up (small degree)
      ArrowDown           - Volume down (small degree)
      Shift-ArrowUp       - Volume up (larger degree)
      Shift-ArrowDown     - Volume down (larger degree)

    Transport:
      J, ArrowLeft        - Seek back 10 seconds
      K, <space>          - Toggle playback
      L, ArrowRight       - Seek forwards 10 seconds
      <, Shift-ArrowLeft  - Previous track
      >, Shift-ArrowRight - Next track

    Additional:
      Shift-L             - Switch to logs details view
      Shift-P             - Switch to payloads details view
-->

<!-- Taken from: https://svelte.dev/repl/48bd3726b74c4329a186838ce645099b?version=3.46.4 -->

<script lang="ts">
    import { activeDetailsView, type DetailsView, DEV_MODE } from "../state.js";
    import preAmpManager, { VolumeChangeDegree } from "../preAmpManager.ts";
    import transportManager from "../transportManager.ts";
    import pubSub from "../pubSub.ts";

    const preAmp = preAmpManager();
    const transport = transportManager();

    let isShiftPressed = false;
    let isSpacePressed = false;
    let isCapitalDPressed = false;
    let isCapitalLPressed = false;
    let isCapitalPPressed = false;
    let isCPressed = false;
    let isIPressed = false;
    let isJPressed = false;
    let isKPressed = false;
    let isLPressed = false;
    let isMPressed = false;
    let isPPressed = false;
    let isQPressed = false;
    let isSPressed = false;
    let isTPressed = false;
    let isLessThanPressed = false;
    let isGreaterThanPressed = false;
    let isArrowDownPressed = false;
    let isArrowLeftPressed = false;
    let isArrowRightPressed = false;
    let isArrowUpPressed = false;

    // --------------------------------------------------------------------------------------------
    // Handlers
    // --------------------------------------------------------------------------------------------

    const showDev = () => DEV_MODE && activeDetailsView.set($activeDetailsView !== "dev" ? "dev" : undefined);

    const showLogs = () => activeDetailsView.set($activeDetailsView !== "logs" ? "logs" : undefined);

    const showPayloads = () => activeDetailsView.set($activeDetailsView !== "payloads" ? "payloads" : undefined);

    const handleDetailsToggle = (detailsView: DetailsView) => {
        $activeDetailsView = $activeDetailsView === detailsView ? undefined : detailsView;
    };

    // --------------------------------------------------------------------------------------------

    const onKeyDown = (event: KeyboardEvent) => {
        // `keydown` event is fired while the physical key is held down.

        // Assuming you only want to handle the first press, we early return to skip.
        if (event.repeat) return;

        switch (event.key) {
            case "Shift":
                isShiftPressed = true;
                event.preventDefault();
                break;
            case " ":
                isSpacePressed = true;
                event.preventDefault();
                break;
            case "c":
                isCPressed = true;
                event.preventDefault();
                break;
            case "D":
                isCapitalDPressed = true;
                event.preventDefault();
                break;
            case "i":
                isIPressed = true;
                event.preventDefault();
                break;
            case "j":
                isJPressed = true;
                event.preventDefault();
                break;
            case "k":
                isKPressed = true;
                event.preventDefault();
                break;
            case "l":
                isLPressed = true;
                event.preventDefault();
                break;
            case "L":
                isCapitalLPressed = true;
                event.preventDefault();
                break;
            case "m":
                isMPressed = true;
                event.preventDefault();
                break;
            case "p":
                isPPressed = true;
                event.preventDefault();
                break;
            case "P":
                isCapitalPPressed = true;
                event.preventDefault();
                break;
            case "q":
                isQPressed = true;
                event.preventDefault();
                break;
            case "s":
                isSPressed = true;
                event.preventDefault();
                break;
            case "t":
                isTPressed = true;
                event.preventDefault();
                break;
            case "<":
                isLessThanPressed = true;
                event.preventDefault();
                break;
            case ">":
                isGreaterThanPressed = true;
                event.preventDefault();
                break;
            case "ArrowDown":
                isArrowDownPressed = true;
                event.preventDefault();
                break;
            case "ArrowLeft":
                isArrowLeftPressed = true;
                event.preventDefault();
                break;
            case "ArrowRight":
                isArrowRightPressed = true;
                event.preventDefault();
                break;
            case "ArrowUp":
                isArrowUpPressed = true;
                event.preventDefault();
                break;
        }

        if (isCapitalDPressed) {
            showDev();
        }

        if (isCapitalLPressed) {
            showLogs();
        }

        if (isCPressed && $activeDetailsView === "queue") {
            pubSub.publish("ScrollToCurrentQueueItem");
        }

        if (isIPressed) {
            handleDetailsToggle("info");
        }

        if (isKPressed || isSpacePressed) {
            transport.togglePlayback();
        }

        if (isMPressed) {
            preAmp.toggleMute();
        }

        if (isPPressed) {
            handleDetailsToggle("presets");
        }

        if (isCapitalPPressed) {
            showPayloads();
        }

        if (isQPressed) {
            handleDetailsToggle("queue");
        }

        if (isSPressed) {
            handleDetailsToggle("sources");
        }

        if (isTPressed) {
            handleDetailsToggle("streamer");
        }

        if (isArrowDownPressed) {
            preAmp.volumeDown(isShiftPressed ? VolumeChangeDegree.Big : VolumeChangeDegree.Small);
        }

        if ((isArrowLeftPressed && !isShiftPressed) || isJPressed) {
            transport.seekBackwards();
        }

        if ((isShiftPressed && isArrowLeftPressed) || isLessThanPressed) {
            transport.previousTrack();
        }

        if ((isArrowRightPressed && !isShiftPressed) || isLPressed) {
            transport.seekForwards();
        }

        if ((isShiftPressed && isArrowRightPressed) || isGreaterThanPressed) {
            transport.nextTrack();
        }

        if (isArrowUpPressed) {
            preAmp.volumeUp(isShiftPressed ? VolumeChangeDegree.Big : VolumeChangeDegree.Small);
        }
    };

    const onKeyUp = (event: KeyboardEvent) => {
        // `keyup` fires whenever the physical key was let go.
        switch (event.key) {
            case "Shift":
                isShiftPressed = false;
                event.preventDefault();
                break;
            case " ":
                isSpacePressed = false;
                event.preventDefault();
                break;
            case "c":
                isCPressed = false;
                event.preventDefault();
                break;
            case "D":
                isCapitalDPressed = false;
                event.preventDefault();
                break;
            case "i":
                isIPressed = false;
                event.preventDefault();
                break;
            case "j":
                isJPressed = false;
                event.preventDefault();
                break;
            case "k":
                isKPressed = false;
                event.preventDefault();
                break;
            case "l":
                isLPressed = false;
                event.preventDefault();
                break;
            case "L":
                isCapitalLPressed = false;
                event.preventDefault();
                break;
            case "m":
                isMPressed = false;
                event.preventDefault();
                break;
            case "p":
                isPPressed = false;
                event.preventDefault();
                break;
            case "P":
                isCapitalPPressed = false;
                event.preventDefault();
                break;
            case "q":
                isQPressed = false;
                event.preventDefault();
                break;
            case "s":
                isSPressed = false;
                event.preventDefault();
                break;
            case "t":
                isTPressed = false;
                event.preventDefault();
                break;
            case "<":
                isLessThanPressed = false;
                event.preventDefault();
                break;
            case ">":
                isGreaterThanPressed = false;
                event.preventDefault();
                break;
            case "ArrowDown":
                isArrowDownPressed = false;
                event.preventDefault();
                break;
            case "ArrowLeft":
                isArrowLeftPressed = false;
                event.preventDefault();
                break;
            case "ArrowRight":
                isArrowRightPressed = false;
                event.preventDefault();
                break;
            case "ArrowUp":
                isArrowUpPressed = false;
                event.preventDefault();
                break;
        }
    };
</script>

<svelte:window
    on:keydown={onKeyDown}
    on:keyup={onKeyUp}
/>