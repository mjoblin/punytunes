<!--
    Taken largely from the Floating UI tutorial: https://floating-ui.com/docs/tutorial
    Adapted to Svelte, and the needs of PunyTunes.

    TODO: The Tooltip can sometimes affect the layout of what it's wrapping. For example,
        a Tooltip around an IconButton can result in some extra whitespace below the button
        (see QueueControls). Ideally this would be fixed.
-->

<script lang="ts">
    import type { Placement } from "@floating-ui/dom";
    import { arrow, computePosition, flip, offset as offsetMiddleware, shift } from "@floating-ui/dom";
    import { onMount } from "svelte";

    export let label: string = "";
    export let placement: Placement = "bottom";
    export let offset: number = 10;

    const visibilityDelay = 1000;

    let tooltippedElement: HTMLElement; // Element receiving the tooltip
    let tooltipElement: HTMLElement; // The tooltip itself
    let arrowElement: HTMLElement; // The tooltip arrow

    let visibleTimeout: number | undefined;
    let isTooltipVisible = false;

    onMount(() => {
        computePosition(
            tooltippedElement,
            tooltipElement,
            {
                placement,
                middleware: [
                    offsetMiddleware(offset),
                    flip(),
                    shift({ padding: 15 }),
                    arrow({ element: arrowElement })
                ]
            }
        ).then(({ x, y, placement, middlewareData }) => {
            tooltipElement && Object.assign(tooltipElement.style, {
                left: `${x}px`,
                top: `${y}px`
            });

            const { x: arrowX, y: arrowY } = middlewareData.arrow;

            const staticSide = {
                top: "bottom",
                right: "left",
                bottom: "top",
                left: "right"
            }[placement.split("-")[0]];

            arrowElement && Object.assign(arrowElement.style, {
                left: arrowX != null ? `${arrowX}px` : "",
                top: arrowY != null ? `${arrowY}px` : "",
                right: "",
                bottom: "",
                [staticSide]: "-4px"
            });
        });
    });

    const update = () => {
        computePosition(tooltippedElement, tooltipElement, {
            // options
        }).then(({ x, y, placement, middlewareData }) => {
            // positioning logic
        });
    };

    const showTooltip = () => {
        visibleTimeout && clearTimeout(visibleTimeout);

        visibleTimeout = setTimeout(() => {
            isTooltipVisible = true;
            update();
        }, visibilityDelay);
    };

    const hideTooltip = () => {
        visibleTimeout && clearTimeout(visibleTimeout);
        isTooltipVisible = false;
        visibleTimeout = undefined;
    };
</script>

<div class="Tooltip">
    <div
        class="tooltipped"
        role="none"
        bind:this={tooltippedElement}
        on:mouseenter={showTooltip}
        on:focus={showTooltip}
        on:mouseleave={hideTooltip}
        on:blur={hideTooltip}
    >
        <slot />
    </div>

    <div
        class="tooltip"
        class:tooltip-visible={isTooltipVisible}
        class:tooltip-invisible={!isTooltipVisible}
        role="tooltip"
        bind:this={tooltipElement}
    >
        <span>{label}</span>
        <div class="arrow" bind:this={arrowElement}></div>
    </div>
</div>

<style>
    .Tooltip {
    }

    .tooltip {
        width: max-content;
        position: absolute;
        top: 0;
        left: 0;
        background: var(--accent-color-dim);
        color: white;
        line-height: 1.3;
        font-size: 0.8rem;
        font-weight: 500;
        padding: 5px 8px 5px 8px;
        border-radius: 4px;
    }

    .tooltip-visible {
        opacity: 1;
        transition: opacity 0.3s ease;
    }

    .tooltip-invisible {
        pointer-events: none;
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .arrow {
        position: absolute;
        background: var(--accent-color-dim);
        width: 8px;
        height: 8px;
        transform: rotate(45deg);
    }
</style>
