<script lang="ts">
    import type { Properties } from "csstype";

    export let url: string | null | undefined;
    export let size: number = 60;
    export let radius: number = 3;
    export let fit: Properties["backgroundSize"] = "contain";

    let img = new Image();
    let isLoading = true;
    let isUnavailable = false;

    // Treat a missing URL as unavailable
    $: if (url) {
        img.src = url;
    } else {
        isLoading = false;
        isUnavailable = true;
    }

    // Set loading/unavailable states based on whether the image was successfully loaded
    img.onload = () => {
        isLoading = false;
        isUnavailable = false;
    };

    img.onerror = () => {
        isLoading = false;
        isUnavailable = true;
    };

    img.onabort = () => {
        isLoading = false;
        isUnavailable = true;
    };

    $: cssVarStyles =
        `--size:${size}px;` +
        `--border-radius:${radius}px;` +
        `--background-image:${!isLoading && !isUnavailable && `url("${url}")`};` +
        `--background-size:${fit};`;
</script>

<div
    class="Art"
    class:is-loading={isLoading}
    class:is-small={size <= 30}
    class:art-unavailable={isUnavailable}
    style={cssVarStyles}
/>

<style>
    .Art {
        overflow: hidden;
        min-width: var(--size);
        min-height: var(--size);
        border-radius: var(--border-radius);
        background-image: var(--background-image);
        background-position: center center;
        background-repeat: no-repeat;
        background-size: var(--background-size);
    }

    @keyframes loadingOscillator {
        0%, 100% {
            opacity: 1.0;
        }
        50% {
            opacity: 0.35;
        }
    }

    .is-loading {
        background-color: var(--background-dim);
        animation: loadingOscillator 2s infinite;
    }

    .art-unavailable {
        display: flex;
        justify-content: center;
        align-items: center;
        background-color: var(--background-mid);

        @media (prefers-color-scheme: light) {
            background-color: var(--background-dim);
        }
    }

    .art-unavailable::after {
        content: "no\00000aart";
        text-transform: uppercase;
        text-align: center;
        color: var(--text-dim);
        font-size: 0.6em;
        font-weight: 500;
    }

    .art-unavailable.is-small::after {
        font-size: 0.5em;
        white-space: break-spaces;
    }
</style>