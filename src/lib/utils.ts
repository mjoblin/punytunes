import * as logger from "tauri-plugin-log-api";

/**
 * Convert a duration in seconds into "hh:mm:ss", without the hh: if it would have been "00:".
 *
 * Returns "--:--" if `duration` is undefined. If `clean` is true, any redundant leading 0's and :'s
 * are removed; and it also strips any trailing ".0*" (i.e. milliseconds). `forceHours` will force
 * prepending "00:" to represent zero hours.
 */
const prettyDuration = (
    duration: number | undefined | null,
    clean: boolean = false,
    withUnits: boolean = false,
    forceHours: boolean = false,
) => {
    const leadingZeros = new RegExp("^00:");

    if (typeof duration === "number") {
        const pretty = new Date(duration * 1000)
            .toISOString()
            .substring(11, 19)
            .replace(leadingZeros, "");

        let cleaned = clean ? pretty.replace(/^0+(:0)?/, "").replace(/\.0+$/, "") : pretty;

        let singleDigitMinutes = false;

        if (cleaned.startsWith(":")) {
            cleaned = `0${cleaned}`;
            singleDigitMinutes = true;
        }

        if (forceHours) {
            cleaned = singleDigitMinutes ? `00:0${cleaned}` : `00:${cleaned}`;
        }

        if (withUnits) {
            // Convert "3:04:14" to "3h:04m:14s"
            const components = cleaned.split(":");
            const units = ["s", "m", "h", "d"];

            return components
                .reverse()
                .reduce(
                    (accumString, component, index) =>
                        `${component}${units[index]}${index === 0 ? "" : ":"}${accumString}`,
                    "",
                );
        } else {
            return cleaned;
        }
    } else {
        return "--:--";
    }
};

export { logger, prettyDuration };
