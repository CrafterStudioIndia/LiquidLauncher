// The launcher's background picture and tint, shared by the window and its settings.
import { writable } from "svelte/store";

export const backgroundImage = writable({ dataUrl: null, fileName: null });

export const tint = writable({
    color: "#0b1020",
    opacity: 0.55,
    blur: 0,
});

// Copies the stored appearance options into the live store.
export function applyAppearance(options) {
    const appearance = options?.appearance ?? {};
    tint.set({
        color: appearance.backgroundColor ?? "#0b1020",
        opacity: appearance.backgroundOpacity ?? 0.55,
        blur: appearance.backgroundBlur ?? 0,
    });
}

export const DEFAULT_BACKGROUND_COLOR = "#0b1020";
export const DEFAULT_BACKGROUND_OPACITY = 0.55;
