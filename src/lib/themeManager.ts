import { derived, get, writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/tauri";
import type { Theme } from "@tauri-apps/api/window";

type ThemeManagerActions = {
    setTheme: (theme: string) => void,
    getCssVar: (varName: string) => string,
}

/**
 * The theme manager is a Svelte writable. It owns:
 *
 * 1. The current active theme name (always "light" or "dark"); `$themeManager.theme`
 * 2. Theme setting (where "auto" is accepted but will be converted to light/dark);
 *      `themeManager.setTheme()`
 * 3. CSS variable evaluation (e.g. convert "--some-var" to "#aabbcc"): `themeManager.getCssVar()`
 *
 * The theme manager assumes `tauri-plugin-theme` is being used by the app.
 */
const createThemeManager = () => {
    const activeTheme = writable<Theme>(); // "light" or "dark"
    const activeUserTheme = writable<string>(); // can also be "auto"
    const computedStyles = writable<CSSStyleDeclaration>();

    const managerActions = writable<ThemeManagerActions>({
        setTheme: (theme: string) => {
            invoke("plugin:theme|set_theme", { theme: theme }).then(() => setCurrentThemeState());
        },
        getCssVar: (varName: string): string => "#ff0000",
    });

    // Derive a single manager state from the two separate state writables
    const managerState = derived(
        [activeTheme, activeUserTheme, computedStyles, managerActions],
        ([$activeTheme, $activeUserTheme, $computedStyles, $managerActions]) => ({
            theme: $activeTheme,
            userTheme: $activeUserTheme,
            $computedStyles,
            ...$managerActions,
        }),
    );

    // Set the theme name and computed styles based on current browser state. Theme name will be
    // either "light" or "dark" ("auto" will be evaluated into one of light/dark).
    const setCurrentThemeState = () => {
        activeTheme.set(
            window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark",
        );

        invoke("plugin:theme|get_theme").then((userTheme) =>
            activeUserTheme.set(userTheme as string),
        );

        let currentComputedStyles = getComputedStyle(document.body);
        computedStyles.set(currentComputedStyles);
    };

    computedStyles.subscribe((styles) => {
        // When the computed styles are updated, reset the getCssVar function.
        const getCssVar = (cssVar: string) => {
            const cssValue = styles.getPropertyValue(cssVar);

            if (cssValue.match(/^#[a-fA-F0-9]{6,8}$/)) {
                return cssValue;
            }

            // Return red if var not found, to draw attention to the likely problem
            return "#FF0000";
        };

        managerActions.update((existingManagerActions) => ({
            ...existingManagerActions,
            getCssVar,
        }));
    })

    setCurrentThemeState();

    return managerState;
};

export default createThemeManager;
