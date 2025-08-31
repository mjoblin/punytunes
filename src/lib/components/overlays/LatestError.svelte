<!--
    Display the latest error received from Rust. New errors will replace any currently-showing
    error.
-->

<script lang="ts">
    import { onDestroy } from "svelte";

    import pubSub from "../../pubSub.ts";

    // Unsubscribe functions to be invoked on destroy.
    let unsubscribeFns: (() => void)[] = [];

    onDestroy(() => {
        unsubscribeFns.forEach((unsubscribeFn) => {
            unsubscribeFn();
        });
    });

    const showDuration = 5000;

    let error: string | undefined;
    let showError = false;
    let timeoutId: number | undefined;

    unsubscribeFns.push(
        pubSub.subscribe("AppErrorLog", (log) => {
            error = log.message;

            if (error) {
                showError = true;
                const previousTimeoutId = timeoutId;
                timeoutId = setTimeout(() => showError = false, showDuration);
                clearTimeout(previousTimeoutId);
            }
        })
    );
</script>

<div class="LatestError" class:visible={showError}>
    {error}
</div>

<style>
    .LatestError {
        pointer-events: none;
        position: absolute;
        width: 80vw;
        top: 20px;
        left: 50%;
        transform: translate(-50%, -0%);
        opacity: 0;
        transition: opacity 0.2s ease-in;

        background-color: #531010;
        border: 2px solid red;
        border-radius: 5px;
        padding: 5px 10px 5px 10px;
    }

    .visible {
        opacity: 1;
        transition: opacity 0.2s ease-in;
    }
</style>