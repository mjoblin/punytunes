<script lang="ts">
    import { themeManager } from "../../state.ts";

    export let radius: number = 50;
    export let rotation: number = 0;
    export let startEndGap: number = 0;
    export let thickness: number = 5;
    export let progress: number = 360;
    export let color: string = "--goldenrod";
    export let trackColor: string = "black";

    // Approach to drawing the arc:
    // http://www.independent-software.com/drawing-progress-arc-in-pure-css-using-skewed-rectangles.html

    const FULL_ARC = 359.999;
    const START_END_OFFSET = startEndGap / 2;
    const MAX_PROGRESS = FULL_ARC - startEndGap;

    // Arc properties. The arc is some segment of a circle (x and y radius are the same). An
    // incomplete circle is created by specifying a non-zero startEndGap (the gap in the circle,
    // in degrees). The progress in the arc is computed to be 0-360 scaled to the length of the
    // arc (360 less the gap).

    $: arcContainerSize = radius * 2;
    $: arcCenterSVG = radius;
    $: radiusSVG = radius - thickness;
    $: progressInArc = progress * (MAX_PROGRESS / FULL_ARC);
    $: progressDisplay = progressInArc < 0 ? 0 : progressInArc >= MAX_PROGRESS ? MAX_PROGRESS : progressInArc;

    $: if (color.startsWith("--")) {
        color = $themeManager.getCssVar(color);
    }

    $: trackColorDisplay = trackColor.startsWith("--")
        ? $themeManager.getCssVar(trackColor)
        : trackColor;

    // SVG: origin (0, 0) is the top left; +x goes right, +y goes down
    // https://stackoverflow.com/questions/5736398/how-to-calculate-the-svg-path-for-an-arc-of-a-circle

    const polarToCartesian = (centerX: number, centerY: number, radius: number, angleInDegrees: number) => {
        var angleInRadians = (angleInDegrees-90) * Math.PI / 180.0;

        return {
            x: centerX + (radius * Math.cos(angleInRadians)),
            y: centerY + (radius * Math.sin(angleInRadians))
        };
    };

    const describeArc = (x: number, y: number, radius: number, startAngle: number, endAngle: number) => {
        var start = polarToCartesian(x, y, radius, endAngle);
        var end = polarToCartesian(x, y, radius, startAngle)

        var largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";

        var d = [
            "M", start.x, start.y,
            "A", radius, radius, 0, largeArcFlag, 0, end.x, end.y
        ].join(" ");

        return d;
    };

    $: cssVarStyles =
        `--arc-container-size:${arcContainerSize}px;` +
        `--thickness:${thickness};` +
        `--progress-color:${color};` +
        `--track-color:${trackColorDisplay}`;
</script>

<div class="Arc" style={cssVarStyles}>
    <svg class="arc-svg" xmlns="http://www.w3.org/2000/svg">
        <g transform="rotate({rotation} {arcCenterSVG} {arcCenterSVG})">
            <path
                class="arc-track"
                d={describeArc(arcCenterSVG, arcCenterSVG, radiusSVG, START_END_OFFSET, START_END_OFFSET + MAX_PROGRESS)}
            />
            <path
                class="arc-progress"
                d={describeArc(arcCenterSVG, arcCenterSVG, radiusSVG, START_END_OFFSET, START_END_OFFSET + progressDisplay)}
            />
        </g>
    </svg>

    <div class="arc-content">
        <slot />
    </div>
</div>

<style>
    .Arc {
        width: var(--arc-container-size);
        height: var(--arc-container-size);
        position: relative;

        & .arc-svg {
            width: 100%;
            height: 100%;
            stroke-width: var(--thickness);
            stroke-linecap: round;
        }
    }

    .arc-track {
        stroke: var(--track-color);
        fill: none;
    }

    .arc-progress {
        stroke: var(--progress-color);
        fill: none;
    }

    .arc-content {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
    }
</style>
